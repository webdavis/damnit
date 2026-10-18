use dam_domain::{
    Blocker, ChildDisposition, Completion, Date, DependencyDisposition, Dispositions, Force,
    Object, Oid, Recurrence, Task, Tracked, When, blockers, roll_forward,
};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, ObjectRepository, Randomness};
use crate::use_cases::atomically::atomically;
use crate::use_cases::subtree::{children, move_into};

pub struct CompletePlan {
    pub oid: Oid,
    pub blockers: Vec<Blocker>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Completed {
    Done,
    RolledForward { next_due: Date },
}

pub fn plan_complete(
    objects: &dyn ObjectRepository,
    oid: &Oid,
) -> Result<CompletePlan, UseCaseError> {
    let task = load_task(objects, oid)?;
    let mut open_deps = Vec::new();
    for dep in &task.base.depends {
        if is_an_open_task(objects, dep)? {
            open_deps.push(dep.clone());
        }
    }
    let open_children: Vec<Oid> = children(objects, &task.base)?
        .iter()
        .filter(|c| c.as_task().is_some_and(|t| !t.completion.is_done()))
        .map(|c| c.oid().clone())
        .collect();
    Ok(CompletePlan {
        oid: oid.clone(),
        blockers: blockers(&open_deps, &open_children),
    })
}

pub fn complete(
    objects: &dyn ObjectRepository,
    clock: &dyn Clock,
    random: &dyn Randomness,
    oid: &Oid,
    force: Force,
    tracked: Tracked,
) -> Result<Completed, UseCaseError> {
    atomically(objects, || {
        let plan = plan_complete(objects, oid)?;
        if !plan.blockers.is_empty() {
            match force {
                Force::No => {
                    return Err(Refusal::Blocked {
                        oid: oid.clone(),
                        blockers: plan.blockers,
                    }
                    .into());
                }
                Force::Yes => {}
                Force::With(d) => apply_dispositions(objects, random, oid, &plan.blockers, d)?,
            }
        }
        let mut task = load_task(objects, oid)?;
        let outcome = match next_due_if_recurring(&task, clock.today(), tracked)? {
            Some(next) => {
                task.due = Some(When::Day(next));
                Completed::RolledForward { next_due: next }
            }
            None => {
                if !task.completion.is_done() {
                    task.completion = Completion::Done {
                        at: Some(clock.now()),
                    };
                }
                Completed::Done
            }
        };
        objects.put(&Object::Task(task))?;
        Ok(outcome)
    })
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
    found: &[Blocker],
    d: Dispositions,
) -> Result<(), UseCaseError> {
    let mut task = load_task(objects, oid)?;
    let open_children: Vec<Oid> = found
        .iter()
        .filter_map(|b| match b {
            Blocker::OpenChild(c) => Some(c.clone()),
            Blocker::OpenDependency(_) => None,
        })
        .collect();
    let deps: Vec<Oid> = found
        .iter()
        .filter_map(|b| match b {
            Blocker::OpenDependency(x) => Some(x.clone()),
            Blocker::OpenChild(_) => None,
        })
        .collect();

    let target = match d.children {
        ChildDisposition::Keep => None,
        ChildDisposition::Up => Some(task.base.path.clone()),
        ChildDisposition::Into(_) if open_children.is_empty() => None,
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
        for child in &open_children {
            move_into(objects, child, &folder)?;
        }
    }
    if d.dependencies == DependencyDisposition::Drop {
        task.base.depends.retain(|x| !deps.contains(x));
        objects.put(&Object::Task(task))?;
    }
    Ok(())
}

fn load_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<Task, UseCaseError> {
    match objects.get(oid)? {
        Some(Object::Task(t)) => Ok(t),
        Some(Object::Event(_)) => Err(Refusal::NotATask(oid.clone()).into()),
        None => Err(Refusal::NoSuchObject(oid.short().to_string()).into()),
    }
}

fn is_an_open_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<bool, UseCaseError> {
    Ok(matches!(objects.get(oid)?, Some(Object::Task(t)) if !t.completion.is_done()))
}

#[cfg(test)]
mod tests;
