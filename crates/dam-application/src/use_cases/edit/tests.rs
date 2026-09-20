use super::*;
use crate::testing::prelude::*;
use crate::testing::{MemoryStore, oid};
use dam_domain::{Categories, Category, Event, Object, Task, When};
use jiff::civil::date;

fn seed(store: &MemoryStore, byte: u8) -> Oid {
    store
        .put(&Object::Task(Task::new(oid(byte), format!("t{byte}"))))
        .unwrap();
    oid(byte)
}

#[test]
fn subject_priority_and_due_are_set() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let out = edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            subject: Some("new".into()),
            priority: Some(Priority::HIGHEST),
            due: Some(Some(When::Day(date(2026, 9, 25)))),
            ..EditFields::default()
        },
    )
    .unwrap();
    let t = out.as_task().unwrap();
    assert_eq!(t.base.subject, "new");
    assert_eq!(t.priority, Priority::HIGHEST);
    assert_eq!(t.due, Some(When::Day(date(2026, 9, 25))));
    assert_eq!(store.get(&id).unwrap(), Some(out));
}

#[test]
fn some_none_clears_a_date() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            due: Some(Some(When::Day(date(2026, 9, 25)))),
            ..Default::default()
        },
    )
    .unwrap();
    let out = edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            due: Some(None),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.as_task().unwrap().due, None);
}

#[test]
fn labels_are_added_removed_and_checked() {
    let cats = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["light".into(), "deep".into()],
        exclusive: true,
    }])
    .unwrap();
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    edit(
        &store,
        &cats,
        &id,
        EditFields {
            add_labels: vec!["deep".into(), "errand".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let err = edit(
        &store,
        &cats,
        &id,
        EditFields {
            add_labels: vec!["light".into()],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, UseCaseError::Refused(Refusal::Labels(_))));
    let out = edit(
        &store,
        &cats,
        &id,
        EditFields {
            remove_labels: vec!["deep".into()],
            add_labels: vec!["light".into()],
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.base().labels.contains("light") && !out.base().labels.contains("deep"));
}

#[test]
fn a_dependency_cycle_is_refused() {
    let store = MemoryStore::new();
    let a = seed(&store, 1);
    let b = seed(&store, 2);
    edit(
        &store,
        &Categories::default(),
        &b,
        EditFields {
            add_depends: vec![a.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    let err = edit(
        &store,
        &Categories::default(),
        &a,
        EditFields {
            add_depends: vec![b.clone()],
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::Cycle {
            oid: a,
            path: vec![b, oid(1)]
        })
    );
}

#[test]
fn a_bad_recurrence_is_a_parse_error_and_a_good_one_is_kept() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let err = edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            recurrence: Some(Some("weekly".into())),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, UseCaseError::Parse(_)));
    let out = edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            recurrence: Some(Some("every! week".into())),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.base().recurrence.as_deref(), Some("every! week"));
}

#[test]
fn attach_needs_an_existing_event() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let err = edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            attach: Some(Some(oid(9))),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(_))
    ));
    store
        .put(&Object::Event(Event::new(
            oid(9),
            "e",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )))
        .unwrap();
    let out = edit(
        &store,
        &Categories::default(),
        &id,
        EditFields {
            attach: Some(Some(oid(9))),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.as_task().unwrap().event, Some(oid(9)));
}

#[test]
fn task_fields_on_an_event_are_refused() {
    let store = MemoryStore::new();
    store
        .put(&Object::Event(Event::new(
            oid(9),
            "e",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )))
        .unwrap();
    let err = edit(
        &store,
        &Categories::default(),
        &oid(9),
        EditFields {
            priority: Some(Priority::HIGHEST),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(9))));
}
