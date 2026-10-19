use super::super::*;
use super::seed;
use crate::testing::prelude::*;
use crate::testing::{Fixture, MemoryStore, oid};
use dam_domain::{Completion, Event, Object, Task, When};
use jiff::civil::date;

#[test]
fn undone_reopens_a_completed_task() {
    let store = MemoryStore::new();
    let mut task = Task::new(oid(1), "t1");
    task.completion = Completion::Done {
        at: Some(jiff::Timestamp::UNIX_EPOCH),
    };
    store.put(&Object::Task(task)).unwrap();
    let id = oid(1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let out = dam
        .edit(
            &id,
            EditFields {
                undone: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(out.as_task().unwrap().completion, Completion::Open);
    assert_eq!(store.get(&id).unwrap(), Some(out));
}

#[test]
fn undone_on_a_task_that_is_not_completed_is_refused_naming_it() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let err = dam
        .edit(
            &id,
            EditFields {
                undone: true,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NotCompleted(id.clone()))
    );
    assert!(err.to_string().contains(id.short()), "{err}");
}

#[test]
fn undone_on_an_event_is_refused_as_not_a_task() {
    let store = MemoryStore::new();
    store
        .put(&Object::Event(Event::new(
            oid(9),
            "e",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )))
        .unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let err = dam
        .edit(
            &oid(9),
            EditFields {
                undone: true,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(9))));
}
