use std::collections::BTreeMap;

use dam_domain::{Object, Oid, OpenChildren, Task};

use crate::config::{HookEvent, OnFailure};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{HookFailure, ObjectRepository, Step};
use crate::service::Dam;
use crate::use_cases::list::log_by_object::LogByObject;
use edit_request::Edited;

impl Dam<'_> {
    pub(crate) fn entering<T>(
        &self,
        completing: &[Oid],
        work: impl FnOnce() -> Result<T, UseCaseError>,
    ) -> Result<T, UseCaseError> {
        if self.access.token().is_some()
            || (self.config.hooks.is_empty() && self.project_hooks.is_none())
        {
            return work();
        }
        let before = tasks_by_oid(self.store)?;
        let produced = work()?;
        let after = tasks_by_oid(self.store)?;
        let children = OpenChildren::of(after.values());
        for (oid, after) in after {
            let earlier = before.get(&oid);
            if earlier == Some(&after) {
                continue;
            }
            let event = match earlier {
                None => HookEvent::TaskNew,
                Some(_) if completing.contains(&oid) => HookEvent::TaskDone,
                Some(was) if !was.completion.is_done() && after.completion.is_done() => {
                    HookEvent::TaskDone
                }
                Some(_) => HookEvent::TaskChanged,
            };
            self.run_hook(event, Object::Task(after), &children)?;
        }
        Ok(produced)
    }

    fn run_hook(
        &self,
        event: HookEvent,
        task: Object,
        children: &OpenChildren,
    ) -> Result<(), UseCaseError> {
        let oid = task.oid().clone();
        self.run_one_hook(event, task, children)?;
        let Some(project) = self.project_hooks else {
            return Ok(());
        };
        match self.store.get(&oid)? {
            Some(now) if now.base().path.is_within(project.within) => Dam {
                config: project.config,
                hooks: project.hooks,
                project_hooks: None,
                ..*self
            }
            .run_one_hook(event, now, children),
            _ => Ok(()),
        }
    }

    fn run_one_hook(
        &self,
        event: HookEvent,
        task: Object,
        children: &OpenChildren,
    ) -> Result<(), UseCaseError> {
        let Some(hook) = self.config.hook(event) else {
            return Ok(());
        };
        let task = self.handed(task, children)?;
        let histories = LogByObject::new(self);
        let step = Step {
            name: event.as_str(),
            remote: None,
            doing: &format!("running {}", event.as_str()),
            did: &format!("ran {}", event.as_str()),
            unit: "",
            total: None,
        };
        let ran = self.waiting_on(step, || self.hooks.run(event, &task, &histories));
        histories.unread()?;
        let failure = match ran {
            Ok(None) => return Ok(()),
            Ok(Some(request)) => match self.hook_edit(task.object.oid(), &request)? {
                Edited::Nothing => return Ok(()),
                Edited::Changed => {
                    self.store.mark_hooked(task.object.oid(), event)?;
                    self.hook_edits.edited(event, task.object.oid());
                    return Ok(());
                }
                Edited::Failed(failure) => failure,
            },
            Err(failure) => failure,
        };
        let failure = HookFailure { event, failure };
        match hook.on_failure() {
            OnFailure::Warn => {
                self.hook_warnings.warn(&failure);
                Ok(())
            }
            OnFailure::Refuse => Err(Refusal::HookRefused(failure).into()),
        }
    }
}

fn tasks_by_oid(objects: &dyn ObjectRepository) -> Result<BTreeMap<Oid, Task>, UseCaseError> {
    Ok(objects
        .all()?
        .into_iter()
        .filter_map(|object| match object {
            Object::Task(task) => Some((task.base.oid.clone(), task)),
            Object::Event(_) => None,
        })
        .collect())
}

mod details;
mod edit_request;
pub(crate) mod listed;
pub(crate) mod pulled;
mod run_time;

#[cfg(test)]
mod tests;
