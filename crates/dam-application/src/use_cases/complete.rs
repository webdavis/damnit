use dam_domain::{
    Completion, Date, Force, Object, Oid, Permission, Recurrence, Task, Tracked, When, roll_forward,
};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::ObjectRepository;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::Landed;
use crate::use_cases::load::load;
use dispositions::apply_dispositions;

pub use blockers::CompletePlan;

#[derive(Debug, PartialEq, Eq)]
pub enum Completed {
    Done,
    RolledForward { next_due: Date },
}

impl Dam<'_> {
    pub fn complete(
        &self,
        oid: &Oid,
        force: Force,
        tracked: Tracked,
    ) -> Result<(Completed, Landed), UseCaseError> {
        atomically(self.store, || {
            if matches!(force, Force::With(_)) && self.access.token().is_some() {
                return Err(Refusal::NotOpenToTokens("done --force").into());
            }
            let plan = self.plan_complete(oid)?;
            if !plan.blockers.is_empty() {
                match force {
                    Force::No => {
                        return Err(Refusal::Blocked {
                            oid: oid.clone(),
                            blockers: plan.blockers,
                        }
                        .into());
                    }
                    Force::With(d) => {
                        apply_dispositions(self.store, self.random, oid, &plan.blockers, d)?
                    }
                }
            }
            let before = load_task(self.store, oid)?;
            let mut task = before.clone();
            let outcome = match next_due_if_recurring(&task, self.clock.today(), tracked)? {
                Some(next) => {
                    task.due = Some(When::Day(next));
                    Completed::RolledForward { next_due: next }
                }
                None => {
                    if !task.completion.is_done() {
                        task.completion = Completion::Done {
                            at: Some(self.clock.now()),
                        };
                    }
                    Completed::Done
                }
            };
            let landed = self.put_or_send(
                Permission::TaskComplete,
                Some(&Object::Task(before)),
                Object::Task(task),
            )?;
            Ok((outcome, landed))
        })
    }
}

fn next_due_if_recurring(
    task: &Task,
    today: Date,
    tracked: Tracked,
) -> Result<Option<Date>, UseCaseError> {
    let rule = match &task.base.recurrence {
        None => return Ok(None),
        Some(Recurrence::Remote(text)) => {
            return Err(Refusal::UnreadableRecurrence {
                rule: text.clone(),
                tracked,
            }
            .into());
        }
        Some(Recurrence::Dam { rule, .. }) => rule,
    };
    let due = task.due.as_ref().map(|d| d.date()).unwrap_or(today);
    Ok(roll_forward(rule, due, today))
}

fn load_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<Task, UseCaseError> {
    match load(objects, oid)? {
        Object::Task(t) => Ok(t),
        Object::Event(_) => Err(Refusal::NotATask(oid.clone()).into()),
    }
}

mod blockers;
mod dispositions;

#[cfg(test)]
mod tests;
