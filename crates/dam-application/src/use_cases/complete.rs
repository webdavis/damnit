use dam_domain::{
    Blockers, ChildDisposition, Completion, Date, DependencyDisposition, Dispositions, Force,
    Object, Oid, Recurrence, Task, Tracked, When, roll_forward,
};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ObjectRepository, Randomness};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::load;
use crate::use_cases::subtree::{children, move_into};

pub struct CompletePlan {
    pub oid: Oid,
    pub blockers: Blockers,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Completed {
    Done,
    RolledForward { next_due: Date },
}

impl Dam<'_> {
    pub fn plan_complete(&self, oid: &Oid) -> Result<CompletePlan, UseCaseError> {
        let task = load_task(self.store, oid)?;
        let mut dependencies = Vec::new();
        for dep in &task.base.depends {
            if is_an_open_task(self.store, dep)? {
                dependencies.push(dep.clone());
            }
        }
        let children: Vec<Oid> = children(self.store, &task.base)?
            .iter()
            .filter(|c| c.as_task().is_some_and(|t| !t.completion.is_done()))
            .map(|c| c.oid().clone())
            .collect();
        Ok(CompletePlan {
            oid: oid.clone(),
            blockers: Blockers {
                dependencies,
                children,
            },
        })
    }

    pub fn complete(
        &self,
        oid: &Oid,
        force: Force,
        tracked: Tracked,
    ) -> Result<Completed, UseCaseError> {
        atomically(self.store, || {
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
            let mut task = load_task(self.store, oid)?;
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
            self.store.put(&Object::Task(task))?;
            Ok(outcome)
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

fn apply_dispositions(
    objects: &dyn ObjectRepository,
    random: &dyn Randomness,
    oid: &Oid,
    found: &Blockers,
    d: Dispositions,
) -> Result<(), UseCaseError> {
    let mut task = load_task(objects, oid)?;

    let target = match d.children {
        ChildDisposition::Keep => None,
        ChildDisposition::Up => Some(task.base.path.clone()),
        ChildDisposition::Into(_) if found.children.is_empty() => None,
        ChildDisposition::Into(name) => {
            let group_oid = Oid::generate(&mut |b| random.fill(b));
            let mut group = Task::new(group_oid, name.clone());
            group.base.path = task.base.path.clone();
            let folder = group
                .base
                .owned_folder()
                .ok_or_else(|| UseCaseError::Parse(format!("{name:?} cannot name a folder")))?;
            objects.put(&Object::Task(group))?;
            Some(folder)
        }
    };
    if let Some(folder) = target {
        for child in &found.children {
            move_into(objects, child, &folder)?;
        }
    }
    if d.dependencies == DependencyDisposition::Drop {
        task.base
            .depends
            .retain(|x| !found.dependencies.contains(x));
        objects.put(&Object::Task(task))?;
    }
    Ok(())
}

fn load_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<Task, UseCaseError> {
    match load(objects, oid)? {
        Object::Task(t) => Ok(t),
        Object::Event(_) => Err(Refusal::NotATask(oid.clone()).into()),
    }
}

fn is_an_open_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<bool, UseCaseError> {
    Ok(matches!(objects.get(oid)?, Some(Object::Task(t)) if !t.completion.is_done()))
}

#[cfg(test)]
mod tests;
