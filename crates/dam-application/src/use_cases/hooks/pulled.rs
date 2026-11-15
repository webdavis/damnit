use dam_domain::{CommitId, Object, Oid};

use super::edit_request::Edited;
use crate::access::Owner;
use crate::config::{HookEvent, PulledChanges};
use crate::errors::UseCaseError;
use crate::ports::{HelperError, HookFailure, Pull, PulledChange, RemoteName};
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
        let Some(hook) = self.config.task_pulled.filter(|_| !pulled.is_empty()) else {
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
                    if run.failed == 0 && self.access.token().is_none() {
                        let event = HookEvent::TaskPulled;
                        self.hook_warnings.warn(&HookFailure { event, failure });
                    }
                    run.failed += 1;
                }
            }
            run.ran += 1;
        }
        run.not_reached = pulled.len() - run.ran;
        if let Some(before) = &before {
            run.commit = self.commit_hook_edits(remote, before)?;
        }
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
mod run_time;

use committed::BeforeTheHook;
use run_time::RunTime;

#[cfg(test)]
mod tests;
