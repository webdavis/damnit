use dam_domain::{Object, Oid};

use super::edit_request::Edited;
use super::run_time::RunTime;
use crate::config::HookEvent;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{
    FunctionFailure, HelperError, HookFailure, Pull, PulledChange, RemoteName, Step,
};
use crate::service::Dam;
use crate::use_cases::list::log_by_object::LogByObject;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedHookRun {
    pub hook: HookEvent,
    pub ran: usize,
    pub changed: usize,
    pub failed: usize,
    pub not_reached: usize,
    pub error: Option<FunctionFailure>,
}

impl Dam<'_> {
    pub fn run_hook_over_listed(
        &self,
        hook: HookEvent,
        query: Option<&str>,
    ) -> Result<ListedHookRun, UseCaseError> {
        if !self.config.declares(hook) {
            return Err(Refusal::HookNotDeclared(hook).into());
        }
        let listed: Vec<Oid> = self
            .list(query)?
            .into_iter()
            .filter_map(|object| match object {
                Object::Task(task) => Some(task.base.oid),
                Object::Event(_) => None,
            })
            .collect();
        let mut run = ListedHookRun {
            hook,
            ran: 0,
            changed: 0,
            failed: 0,
            not_reached: 0,
            error: None,
        };
        let run_time = RunTime::started_on(self.clock);
        let progress = self.shown_progress();
        progress.start(Step {
            name: hook.as_str(),
            remote: None,
            doing: &format!("running {}", hook.as_str()),
            did: &format!("ran {}", hook.as_str()),
            unit: "tasks",
            total: Some(listed.len()),
        });
        for (looked_at, oid) in listed.iter().enumerate() {
            if run_time.ran_out() || self.interruption.requested() {
                break;
            }
            if let Some(called) = self.listed_hook_on(hook, oid)? {
                self.count(&mut run, oid, called)?;
            }
            progress.advance(looked_at + 1);
        }
        progress.finish();
        run.not_reached = listed.len() - run.ran;
        if self.interruption.requested() {
            return Err(HelperError::Cancelled.into());
        }
        Ok(run)
    }

    fn count(
        &self,
        run: &mut ListedHookRun,
        oid: &Oid,
        called: Edited,
    ) -> Result<(), UseCaseError> {
        run.ran += 1;
        match called {
            Edited::Nothing => {}
            Edited::Changed => {
                run.changed += 1;
                self.store.mark_hooked(oid, run.hook)?;
                self.hook_edits.edited(run.hook, oid);
            }
            Edited::Failed(failure) => {
                run.failed += 1;
                if run.error.is_none() {
                    let event = run.hook;
                    let failure = failure.clone();
                    self.hook_warnings
                        .warn_change_dropped(&HookFailure { event, failure });
                }
                run.error.get_or_insert(failure);
            }
        }
        Ok(())
    }

    fn listed_hook_on(&self, hook: HookEvent, oid: &Oid) -> Result<Option<Edited>, UseCaseError> {
        let Some(task) = self.store.get(oid)? else {
            return Ok(Some(Edited::Nothing));
        };
        let answer = match hook {
            HookEvent::TaskPulled => {
                let Some(remote) = self.tracking_remote(oid)? else {
                    return Ok(None);
                };
                let change = PulledChange::New;
                self.hooks.pulled(&task, None, &Pull { remote, change })
            }
            local => {
                let histories = LogByObject::new(self);
                let answer = self.hooks.run(local, &task, &histories);
                histories.unread()?;
                answer
            }
        };
        match answer {
            Ok(None) => Ok(Some(Edited::Nothing)),
            Ok(Some(request)) => self.hook_edit(oid, &request).map(Some),
            Err(failure) => Ok(Some(Edited::Failed(failure))),
        }
    }

    fn tracking_remote(&self, oid: &Oid) -> Result<Option<RemoteName>, UseCaseError> {
        for remote in &self.config.remotes {
            if self.store.remote_id(&remote.name, oid)?.is_some() {
                return Ok(Some(remote.name.clone()));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
