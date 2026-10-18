mod children;
mod dispositions;

use super::*;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
use dam_domain::{Event, Object, Path, Task, When};
use jiff::civil::date;

pub(super) fn put_task(
    store: &MemoryStore,
    byte: u8,
    subject: &str,
    path: &str,
    done: bool,
) -> Oid {
    let mut t = Task::new(oid(byte), subject);
    t.base.path = Path::parse(path).unwrap();
    t.done = done;
    store.put(&Object::Task(t)).unwrap();
    oid(byte)
}

pub(super) fn path_of(store: &MemoryStore, id: &Oid) -> String {
    store
        .get(id)
        .unwrap()
        .unwrap()
        .base()
        .path
        .as_str()
        .to_string()
}

fn task_of(store: &MemoryStore, id: &Oid) -> Task {
    store.get(id).unwrap().unwrap().as_task().unwrap().clone()
}

pub(super) fn today() -> FixedClock {
    FixedClock(date(2026, 9, 18))
}

#[test]
fn an_unblocked_task_is_done() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    assert!(plan_complete(&store, &id).unwrap().blockers.is_empty());
    assert_eq!(
        complete(&store, &today(), &FixedRandom::new(9), &id, Force::No).unwrap(),
        Completed::Done
    );
    assert!(task_of(&store, &id).done);
}

#[test]
fn a_recurring_task_rolls_forward_instead_of_closing() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "water the plants", "", false);
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
    let t = task_of(&store, &id);
    assert!(!t.done);
    assert_eq!(t.due, Some(When::Day(date(2026, 9, 25))));
}

#[test]
fn a_rule_dam_cannot_read_is_refused_and_the_task_stays_open() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "water the plants", "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().recurrence = Some("every other week".into());
    store.put(&t).unwrap();
    let err = complete(&store, &today(), &FixedRandom::new(9), &id, Force::No).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::UnreadableRecurrence("every other week".into()))
    );
    assert_eq!(
        err.to_string(),
        "cannot roll \"every other week\" forward; complete it in Todoist or set --recurrence"
    );
    assert!(!task_of(&store, &id).done);
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
fn completing_a_task_records_when_it_happened() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    let clock = FixedClock(date(2026, 9, 18));
    assert_eq!(task_of(&store, &id).completed_at, None);
    complete(&store, &clock, &FixedRandom::new(1), &id, Force::No).unwrap();
    let task = task_of(&store, &id);
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
    let task = task_of(&store, &oid(1));
    assert!(!task.done);
    assert_eq!(task.completed_at, None);
}

#[test]
fn completing_an_already_completed_task_changes_nothing() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
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
    let id = put_task(&store, 1, "milk", "", false);
    let mut t = store.get(&id).unwrap().unwrap();
    t.base_mut().depends.push(oid(3));
    store.put(&t).unwrap();
    assert!(plan_complete(&store, &id).unwrap().blockers.is_empty());
}
