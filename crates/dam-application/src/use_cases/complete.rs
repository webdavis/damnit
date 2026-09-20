use dam_domain::{
    Blocker, ChildDisposition, Date, DependencyDisposition, Force, Object, Oid, Path, Rule, Task,
    When, blockers, roll_forward,
};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, ObjectRepository, Randomness};
use crate::use_cases::subtree::move_subtree;

pub struct CompletePlan {
    pub oid: Oid,
    pub blockers: Vec<Blocker>,
}

/// What the cli asks the user when `Force::Interactive` meets blockers.
pub struct Dispositions {
    pub children: ChildDisposition,
    pub dependencies: DependencyDisposition,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Completed {
    Done,
    RolledForward { next_due: Date },
}

/// Step one: find out whether `oid` is blocked. The cli decides whether to ask.
pub fn plan_complete(
    objects: &dyn ObjectRepository,
    oid: &Oid,
) -> Result<CompletePlan, UseCaseError> {
    let task = load_task(objects, oid)?;
    let mut open_deps = Vec::new();
    for dep in &task.base.depends {
        if is_open(objects, dep)? {
            open_deps.push(dep.clone());
        }
    }
    let open_children: Vec<Oid> = objects
        .children_of(&task.base.path)?
        .iter()
        .filter(|c| c.as_task().is_some_and(|t| !t.done))
        .map(|c| c.oid().clone())
        .collect();
    Ok(CompletePlan {
        oid: oid.clone(),
        blockers: blockers(&open_deps, &open_children),
    })
}

/// Step two: do it. `dispositions` is `None` unless the cli asked.
pub fn complete(
    objects: &dyn ObjectRepository,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
    oid: &Oid,
    force: Force,
    dispositions: Option<Dispositions>,
) -> Result<Completed, UseCaseError> {
    let plan = plan_complete(objects, oid)?;
    if !plan.blockers.is_empty() {
        match (force, dispositions) {
            (Force::No, _) => {
                return Err(Refusal::Blocked {
                    oid: oid.clone(),
                    blockers: plan.blockers,
                }
                .into());
            }
            (Force::Yes, _) | (Force::Interactive, None) => {}
            (Force::Interactive, Some(d)) => {
                apply_dispositions(objects, random, oid, &plan.blockers, d)?;
            }
        }
    }
    let mut task = load_task(objects, oid)?;
    let outcome = match rolled(&task, clock.today()) {
        Some(next) => {
            task.due = Some(When::Day(next));
            Completed::RolledForward { next_due: next }
        }
        None => {
            task.done = true;
            Completed::Done
        }
    };
    objects.put(&Object::Task(task))?;
    Ok(outcome)
}

/// The next due date if `task` recurs, else `None` (done like any other task).
fn rolled(task: &Task, today: Date) -> Option<Date> {
    let rule = Rule::parse(task.base.recurrence.as_deref()?).ok()?;
    let due = task.due.as_ref().map(|d| d.date()).unwrap_or(today);
    roll_forward(&rule, due, today)
}

fn apply_dispositions(
    objects: &dyn ObjectRepository,
    random: &mut dyn Randomness,
    oid: &Oid,
    found: &[Blocker],
    d: Dispositions,
) -> Result<(), UseCaseError> {
    let mut task = load_task(objects, oid)?;
    let parent_path = task.base.path.parent().unwrap_or_default();
    let children: Vec<Oid> = found
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
        ChildDisposition::Up => Some(parent_path.clone()),
        ChildDisposition::Into(_) if children.is_empty() => None,
        ChildDisposition::Into(name) => {
            let group_oid = Oid::generate(&mut |b| random.fill(b));
            let mut group = Task::new(group_oid, name.clone());
            group.base.path = parent_path
                .join(&name)
                .map_err(|e| UseCaseError::Parse(e.to_string()))?;
            let path = group.base.path.clone();
            objects.put(&Object::Task(group))?;
            Some(path)
        }
    };
    if let Some(target) = target {
        for child in &children {
            if let Some(c) = objects.get(child)? {
                let segment = last_segment(&c.base().path);
                let to = target
                    .join(&segment)
                    .map_err(|e| UseCaseError::Parse(e.to_string()))?;
                move_subtree(objects, child, &to)?;
            }
        }
    }
    if d.dependencies == DependencyDisposition::Drop {
        task.base.depends.retain(|x| !deps.contains(x));
        objects.put(&Object::Task(task))?;
    }
    Ok(())
}

fn last_segment(path: &Path) -> String {
    path.as_str()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

fn load_task(objects: &dyn ObjectRepository, oid: &Oid) -> Result<Task, UseCaseError> {
    match objects.get(oid)? {
        Some(Object::Task(t)) => Ok(t),
        Some(Object::Event(_)) => Err(Refusal::NotATask(oid.clone()).into()),
        None => Err(Refusal::NoSuchObject(oid.short().to_string()).into()),
    }
}

/// A missing dependency is treated as closed; a failed read propagates, since
/// silently treating it as closed could let a blocked task complete.
fn is_open(objects: &dyn ObjectRepository, oid: &Oid) -> Result<bool, UseCaseError> {
    Ok(matches!(objects.get(oid)?, Some(Object::Task(t)) if !t.done))
}

#[cfg(test)]
mod tests;
