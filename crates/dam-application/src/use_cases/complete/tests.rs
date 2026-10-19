mod children;
mod dispositions;
mod recurrence;

use super::*;
use crate::ports::Clock;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, Fixture, MemoryStore, oid};
use dam_domain::{Completion, Event, Object, Path, Task, Tracked, When};
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
    t.completion = if done {
        Completion::Done { at: None }
    } else {
        Completion::Open
    };
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

pub(super) fn task_of(store: &MemoryStore, id: &Oid) -> Task {
    store.get(id).unwrap().unwrap().as_task().unwrap().clone()
}

pub(super) fn today() -> FixedClock {
    FixedClock(date(2026, 9, 18))
}

#[test]
fn an_unblocked_task_is_done() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    assert!(dam.plan_complete(&id).unwrap().blockers.is_empty());
    assert_eq!(
        dam.complete(&id, Force::No, Tracked::No).unwrap(),
        Completed::Done
    );
    assert!(task_of(&store, &id).completion.is_done());
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
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    let err = dam.complete(&oid(5), Force::No, Tracked::No).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(5))));
}

#[test]
fn completing_a_task_records_when_it_happened() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    let clock = FixedClock(date(2026, 9, 18));
    assert_eq!(task_of(&store, &id).completion, Completion::Open);
    let fixture = Fixture {
        random: FixedRandom::new(1),
        ..Fixture::default()
    };
    let dam = Dam {
        clock: &clock,
        ..fixture.dam(&store)
    };
    dam.complete(&id, Force::No, Tracked::No).unwrap();
    let task = task_of(&store, &id);
    assert_eq!(
        task.completion,
        Completion::Done {
            at: Some(clock.now())
        }
    );
}

#[test]
fn completing_an_already_completed_task_changes_nothing() {
    let store = MemoryStore::new();
    let id = put_task(&store, 1, "milk", "", false);
    let first = FixedClock(date(2026, 9, 18));
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    Dam {
        clock: &first,
        random: &FixedRandom::new(1),
        ..dam
    }
    .complete(&id, Force::No, Tracked::No)
    .unwrap();
    let after_first = store.get(&id).unwrap().unwrap();

    let later = FixedClock(date(2026, 9, 19));
    Dam {
        clock: &later,
        random: &FixedRandom::new(1),
        ..dam
    }
    .complete(&id, Force::No, Tracked::No)
    .unwrap();
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
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    assert!(dam.plan_complete(&id).unwrap().blockers.is_empty());
}
