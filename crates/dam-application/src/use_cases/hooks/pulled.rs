use dam_domain::{CommitId, Object, Oid};

use super::edit_request::Edited;
use crate::access::Owner;
use crate::config::{HookEvent, PulledChanges};
use crate::errors::UseCaseError;
use crate::ports::{HelperError, HookFailure, Pull, PulledChange, RemoteName, Step};
use crate::service::Dam;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PulledHookRun {
    pub ran: usize,
    pub changed: usize,
    pub failed: usize,
    pub not_reached: usize,
    pub commit: Option<CommitId>,
}

pub(crate) struct PulledTask {
    oid: Oid,
    before: Option<Object>,
    change: PulledChange,
}

impl PulledTask {
    pub(crate) fn of(before: Option<Object>, theirs: &Object) -> Option<PulledTask> {
        let Object::Task(after) = theirs else {
            return None;
        };
        let change = match &before {
            None => PulledChange::New,
            Some(Object::Task(was)) if !was.completion.is_done() && after.completion.is_done() => {
                PulledChange::Done
            }
            Some(_) => PulledChange::Changed,
        };
        Some(PulledTask {
            oid: after.base.oid.clone(),
            before,
            change,
        })
    }
}

impl Dam<'_> {
    pub(crate) fn run_pulled_hook(
        &self,
        remote: &RemoteName,
        pulled: Vec<PulledTask>,
    ) -> Result<Option<PulledHookRun>, UseCaseError> {
        let Some(hook) = self
            .config
            .task_pulled
            .as_ref()
            .filter(|_| !pulled.is_empty())
        else {
            return Ok(None);
        };
        let before = match hook.changes() {
            PulledChanges::Committed => Some(BeforeTheHook::of(self.store)?),
            PulledChanges::Working => None,
        };
        let as_owner = Dam {
            access: &Owner,
            ..*self
        };
        let mut run = PulledHookRun::default();
        let run_time = RunTime::started_on(self.clock);
        let owners = self.access.token().is_none();
        let progress = self.shown_progress();
        progress.start(Step {
            name: HookEvent::TaskPulled.as_str(),
            remote: Some(&remote.0),
            doing: "running task_pulled",
            did: "ran task_pulled",
            unit: "tasks",
            total: Some(pulled.len()),
        });
        let mut first_error = None;
        for task in &pulled {
            if run_time.ran_out() || self.interruption.requested() {
                break;
            }
            let pull = Pull {
                remote: remote.clone(),
                change: task.change,
            };
            match as_owner.pulled_hook_on(task, &pull)? {
                Edited::Nothing => {}
                Edited::Changed => run.changed += 1,
                Edited::Failed(failure) => {
                    run.failed += 1;
                    if first_error.is_none() && owners {
                        let event = HookEvent::TaskPulled;
                        let failure = failure.clone();
                        self.hook_warnings.warn(&HookFailure { event, failure });
                    }
                    first_error.get_or_insert(failure);
                }
            }
            run.ran += 1;
            progress.advance(run.ran);
        }
        progress.finish();
        run.not_reached = pulled.len() - run.ran;
        if let Some(before) = &before {
            run.commit = self.commit_hook_edits(remote, before)?;
        }
        self.note_pulled_hook_run(remote, &run, first_error)?;
        if self.interruption.requested() {
            return Err(HelperError::Cancelled.into());
        }
        Ok(Some(run))
    }

    fn pulled_hook_on(&self, task: &PulledTask, pull: &Pull) -> Result<Edited, UseCaseError> {
        let Some(now) = self.store.get(&task.oid)? else {
            return Ok(Edited::Nothing);
        };
        match self.hooks.pulled(&now, task.before.as_ref(), pull) {
            Ok(None) => Ok(Edited::Nothing),
            Ok(Some(request)) => self.hook_edit(&task.oid, &request),
            Err(failure) => Ok(Edited::Failed(failure)),
        }
    }
}

mod committed;
mod notice;
mod run_time;

use committed::BeforeTheHook;
use run_time::RunTime;

#[cfg(test)]
mod tests;
