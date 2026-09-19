use dam_domain::{
    Blocker, ChildDisposition, Date, DependencyDisposition, Force, Object, Oid, Path, Rule, Task,
    When, blockers, roll_forward,
};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, ObjectStore, Randomness};

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
pub fn plan_complete(store: &dyn ObjectStore, oid: &Oid) -> Result<CompletePlan, UseCaseError> {
    let task = load_task(store, oid)?;
    let open_deps: Vec<Oid> = task
        .base
        .depends
        .iter()
        .filter(|d| is_open(store, d))
        .cloned()
        .collect();
    let open_children: Vec<Oid> = store
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
    store: &dyn ObjectStore,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
    oid: &Oid,
    force: Force,
    dispositions: Option<Dispositions>,
) -> Result<Completed, UseCaseError> {
    let plan = plan_complete(store, oid)?;
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
                apply_dispositions(store, random, oid, &plan.blockers, d)?;
            }
        }
    }
    let mut task = load_task(store, oid)?;
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
    store.put(&Object::Task(task))?;
    Ok(outcome)
}

/// The next due date if `task` recurs, else `None` (done like any other task).
fn rolled(task: &Task, today: Date) -> Option<Date> {
    let rule = Rule::parse(task.base.recurrence.as_deref()?).ok()?;
    let due = task.due.as_ref().map(|d| d.date()).unwrap_or(today);
    roll_forward(&rule, due, today)
}

fn apply_dispositions(
    store: &dyn ObjectStore,
    random: &mut dyn Randomness,
    oid: &Oid,
    found: &[Blocker],
    d: Dispositions,
) -> Result<(), UseCaseError> {
    let mut task = load_task(store, oid)?;
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
        ChildDisposition::Into(name) => {
            let group_oid = Oid::generate(&mut |b| random.fill(b));
            let mut group = Task::new(group_oid, name.clone());
            group.base.path = parent_path
                .join(&name)
                .map_err(|e| UseCaseError::Parse(e.to_string()))?;
            let path = group.base.path.clone();
            store.put(&Object::Task(group))?;
            Some(path)
        }
    };
    if let Some(target) = target {
        for child in children {
            if let Some(mut c) = store.get(&child)? {
                let segment = last_segment(&c.base().path);
                c.base_mut().path = target
                    .join(&segment)
                    .map_err(|e| UseCaseError::Parse(e.to_string()))?;
                store.put(&c)?;
            }
        }
    }
    if d.dependencies == DependencyDisposition::Drop {
        task.base.depends.retain(|x| !deps.contains(x));
        store.put(&Object::Task(task))?;
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

fn load_task(store: &dyn ObjectStore, oid: &Oid) -> Result<Task, UseCaseError> {
    match store.get(oid)? {
        Some(Object::Task(t)) => Ok(t),
        Some(Object::Event(_)) => Err(Refusal::NotATask(oid.clone()).into()),
        None => Err(Refusal::NoSuchObject(oid.short().to_string()).into()),
    }
}

/// A missing or errored dependency is treated as closed; existence is another
/// use case's concern, not this one's.
fn is_open(store: &dyn ObjectStore, oid: &Oid) -> bool {
    matches!(store.get(oid), Ok(Some(Object::Task(t))) if !t.done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
    use dam_domain::{Blocker, Event, Object, Path, Task, When};
    use jiff::civil::date;

    fn put_task(store: &MemoryStore, byte: u8, path: &str, done: bool) -> Oid {
        let mut t = Task::new(oid(byte), format!("t{byte}"));
        t.base.path = Path::parse(path).unwrap();
        t.done = done;
        store.put(&Object::Task(t)).unwrap();
        oid(byte)
    }

    fn today() -> FixedClock {
        FixedClock(date(2026, 9, 18))
    }

    #[test]
    fn an_unblocked_task_is_done() {
        let store = MemoryStore::new();
        let id = put_task(&store, 1, "", false);
        assert!(plan_complete(&store, &id).unwrap().blockers.is_empty());
        assert_eq!(
            complete(&store, &today(), &mut FixedRandom(9), &id, Force::No, None).unwrap(),
            Completed::Done
        );
        assert!(store.get(&id).unwrap().unwrap().as_task().unwrap().done);
    }

    #[test]
    fn open_children_and_dependencies_block_and_force_no_refuses() {
        let store = MemoryStore::new();
        let parent = put_task(&store, 1, "p", false);
        let child = put_task(&store, 2, "p/c", false);
        let dep = put_task(&store, 3, "", false);
        let mut p = store.get(&parent).unwrap().unwrap();
        p.base_mut().depends.push(dep.clone());
        store.put(&p).unwrap();
        let plan = plan_complete(&store, &parent).unwrap();
        assert_eq!(
            plan.blockers,
            vec![
                Blocker::OpenDependency(dep.clone()),
                Blocker::OpenChild(child.clone())
            ]
        );
        let err = complete(
            &store,
            &today(),
            &mut FixedRandom(9),
            &parent,
            Force::No,
            None,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            UseCaseError::Refused(Refusal::Blocked { .. })
        ));
        assert!(!store.get(&parent).unwrap().unwrap().as_task().unwrap().done);
    }

    #[test]
    fn done_children_do_not_block() {
        let store = MemoryStore::new();
        let parent = put_task(&store, 1, "p", false);
        put_task(&store, 2, "p/c", true);
        assert!(plan_complete(&store, &parent).unwrap().blockers.is_empty());
    }

    #[test]
    fn force_yes_completes_and_leaves_children_where_they_are() {
        let store = MemoryStore::new();
        let parent = put_task(&store, 1, "p", false);
        let child = put_task(&store, 2, "p/c", false);
        complete(
            &store,
            &today(),
            &mut FixedRandom(9),
            &parent,
            Force::Yes,
            None,
        )
        .unwrap();
        assert!(store.get(&parent).unwrap().unwrap().as_task().unwrap().done);
        assert_eq!(
            store.get(&child).unwrap().unwrap().base().path.as_str(),
            "p/c/"
        );
    }

    #[test]
    fn interactive_up_moves_children_beside_the_parent() {
        let store = MemoryStore::new();
        let parent = put_task(&store, 1, "top/p", false);
        let child = put_task(&store, 2, "top/p/c", false);
        let d = Dispositions {
            children: ChildDisposition::Up,
            dependencies: DependencyDisposition::Keep,
        };
        complete(
            &store,
            &today(),
            &mut FixedRandom(9),
            &parent,
            Force::Interactive,
            Some(d),
        )
        .unwrap();
        assert_eq!(
            store.get(&child).unwrap().unwrap().base().path.as_str(),
            "top/c/"
        );
    }

    #[test]
    fn interactive_into_groups_children_under_a_new_task() {
        let store = MemoryStore::new();
        let parent = put_task(&store, 1, "p", false);
        let child = put_task(&store, 2, "p/c", false);
        let d = Dispositions {
            children: ChildDisposition::Into("leftovers".into()),
            dependencies: DependencyDisposition::Keep,
        };
        complete(
            &store,
            &today(),
            &mut FixedRandom(9),
            &parent,
            Force::Interactive,
            Some(d),
        )
        .unwrap();
        let group = store.get(&oid(9)).unwrap().unwrap();
        assert_eq!(group.base().subject, "leftovers");
        assert_eq!(group.base().path.as_str(), "leftovers/");
        assert_eq!(
            store.get(&child).unwrap().unwrap().base().path.as_str(),
            "leftovers/c/"
        );
    }

    #[test]
    fn interactive_drop_removes_open_dependencies() {
        let store = MemoryStore::new();
        let id = put_task(&store, 1, "", false);
        let dep = put_task(&store, 3, "", false);
        let mut t = store.get(&id).unwrap().unwrap();
        t.base_mut().depends.push(dep);
        store.put(&t).unwrap();
        let d = Dispositions {
            children: ChildDisposition::Keep,
            dependencies: DependencyDisposition::Drop,
        };
        complete(
            &store,
            &today(),
            &mut FixedRandom(9),
            &id,
            Force::Interactive,
            Some(d),
        )
        .unwrap();
        let t = store.get(&id).unwrap().unwrap();
        assert!(t.base().depends.is_empty() && t.as_task().unwrap().done);
    }

    #[test]
    fn a_recurring_task_rolls_forward_instead_of_closing() {
        let store = MemoryStore::new();
        let id = put_task(&store, 1, "", false);
        let mut t = store.get(&id).unwrap().unwrap();
        t.base_mut().recurrence = Some("every week".into());
        if let Object::Task(task) = &mut t {
            task.due = Some(When::Day(date(2026, 9, 18)));
        }
        store.put(&t).unwrap();
        let out = complete(&store, &today(), &mut FixedRandom(9), &id, Force::No, None).unwrap();
        assert_eq!(
            out,
            Completed::RolledForward {
                next_due: date(2026, 9, 25)
            }
        );
        let t = store.get(&id).unwrap().unwrap();
        assert!(!t.as_task().unwrap().done);
        assert_eq!(t.as_task().unwrap().due, Some(When::Day(date(2026, 9, 25))));
    }

    #[test]
    fn an_event_is_not_a_task() {
        let store = MemoryStore::new();
        let e = Event::new(
            oid(5),
            "e",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        );
        store.put(&Object::Event(e)).unwrap();
        let err = complete(
            &store,
            &today(),
            &mut FixedRandom(9),
            &oid(5),
            Force::No,
            None,
        )
        .unwrap_err();
        assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(5))));
    }
}
