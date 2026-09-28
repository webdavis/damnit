mod dispositions;

use super::*;
use crate::ports::StoreError;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
use dam_domain::{Blocker, Event, Object, Path, Task, When};
use jiff::civil::date;

struct FailingGetStore {
    objects: Vec<Object>,
    fails_for: Oid,
}

impl ObjectRepository for FailingGetStore {
    fn get(&self, oid: &Oid) -> Result<Option<Object>, StoreError> {
        if *oid == self.fails_for {
            return Err(StoreError::Failed("boom".into()));
        }
        Ok(self.objects.iter().find(|o| o.oid() == oid).cloned())
    }
    fn children_of(&self, _path: &Path) -> Result<Vec<Object>, StoreError> {
        Ok(Vec::new())
    }
    fn all(&self) -> Result<Vec<Object>, StoreError> {
        unimplemented!()
    }
    fn dependents_of(&self, _oid: &Oid) -> Result<Vec<Oid>, StoreError> {
        unimplemented!()
    }
    fn put(&self, _object: &Object) -> Result<(), StoreError> {
        unimplemented!()
    }
    fn delete(&self, _oid: &Oid) -> Result<(), StoreError> {
        unimplemented!()
    }
    fn committed(&self, _oid: &Oid) -> Result<Option<Object>, StoreError> {
        unimplemented!()
    }
}

pub(super) fn put_task(store: &MemoryStore, byte: u8, path: &str, done: bool) -> Oid {
    let mut t = Task::new(oid(byte), format!("t{byte}"));
    t.base.path = Path::parse(path).unwrap();
    t.done = done;
    store.put(&Object::Task(t)).unwrap();
    oid(byte)
}

pub(super) fn today() -> FixedClock {
    FixedClock(date(2026, 9, 18))
}

#[test]
fn an_unblocked_task_is_done() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "", false);
    assert!(plan_complete(&store, &id).unwrap().blockers.is_empty());
    assert_eq!(
        complete(&store, &today(), &FixedRandom::new(9), &id, Force::No).unwrap(),
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
    let err = complete(&store, &today(), &FixedRandom::new(9), &parent, Force::No).unwrap_err();
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
    complete(&store, &today(), &FixedRandom::new(9), &parent, Force::Yes).unwrap();
    assert!(store.get(&parent).unwrap().unwrap().as_task().unwrap().done);
    assert_eq!(
        store.get(&child).unwrap().unwrap().base().path.as_str(),
        "p/c/"
    );
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
    let out = complete(&store, &today(), &FixedRandom::new(9), &id, Force::No).unwrap();
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
    let err = complete(&store, &today(), &FixedRandom::new(9), &oid(5), Force::No).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(5))));
}

#[test]
fn a_store_error_while_checking_a_dependency_propagates() {
    let id = oid(1);
    let dep = oid(3);
    let mut owner = Task::new(id.clone(), "t1");
    owner.base.depends.push(dep.clone());
    let store = FailingGetStore {
        objects: vec![Object::Task(owner)],
        fails_for: dep,
    };
    assert!(matches!(
        plan_complete(&store, &id),
        Err(UseCaseError::Store(_))
    ));
}

#[test]
fn completing_a_task_records_when_it_happened() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "", false);
    let clock = FixedClock(date(2026, 9, 18));
    assert_eq!(
        store
            .get(&id)
            .unwrap()
            .unwrap()
            .as_task()
            .unwrap()
            .completed_at,
        None
    );
    complete(&store, &clock, &FixedRandom::new(1), &id, Force::No).unwrap();
    let task = store.get(&id).unwrap().unwrap();
    let task = task.as_task().unwrap();
    assert!(task.done);
    assert_eq!(task.completed_at, Some(clock.now()));
}

#[test]
fn a_task_that_rolls_forward_records_no_completion_time() {
    let store = MemoryStore::new();
    let mut t = Task::new(oid(1), "water the plants");
    t.base.recurrence = Some("every day".into());
    t.due = Some(When::Day(date(2026, 9, 18)));
    store.put(&Object::Task(t)).unwrap();
    complete(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        &oid(1),
        Force::No,
    )
    .unwrap();
    let task = store.get(&oid(1)).unwrap().unwrap();
    let task = task.as_task().unwrap();
    assert!(!task.done);
    assert_eq!(task.completed_at, None);
}

#[test]
fn completing_an_already_completed_task_changes_nothing() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "", false);
    let first = FixedClock(date(2026, 9, 18));
    complete(&store, &first, &FixedRandom::new(1), &id, Force::No).unwrap();
    let after_first = store.get(&id).unwrap().unwrap();

    let later = FixedClock(date(2026, 9, 19));
    complete(&store, &later, &FixedRandom::new(1), &id, Force::No).unwrap();
    assert_eq!(
        store.get(&id).unwrap().unwrap(),
        after_first,
        "a second done restamped the completion time"
    );
}

#[test]
fn a_dependency_that_no_longer_exists_counts_as_closed() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().depends.push(oid(3));
    store.put(&t).unwrap();
    assert!(plan_complete(&store, &id).unwrap().blockers.is_empty());
}
