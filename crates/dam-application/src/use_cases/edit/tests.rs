mod faults;
mod labels;
mod undone;

use super::*;
use crate::testing::prelude::*;
use crate::testing::{Scenario, event, oid, task};
use dam_domain::When;
use jiff::civil::date;

#[test]
fn subject_priority_and_due_are_set() {
    let s = Scenario::default();
    s.working(task(1));
    let id = oid(1);
    let dam = s.dam();
    let out = dam
        .edit(
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
    assert_eq!(s.store.get(&id).unwrap(), Some(out));
}

#[test]
fn some_none_clears_a_date() {
    let s = Scenario::default();
    s.working(task(1));
    let id = oid(1);
    let dam = s.dam();
    dam.edit(
        &id,
        EditFields {
            due: Some(Some(When::Day(date(2026, 9, 25)))),
            ..Default::default()
        },
    )
    .unwrap();
    let out = dam
        .edit(
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
fn a_dependency_cycle_is_refused() {
    let s = Scenario::default();
    s.working(task(1)).working(task(2));
    let (a, b) = (oid(1), oid(2));
    let dam = s.dam();
    dam.edit(
        &b,
        EditFields {
            add_depends: vec![a.clone()],
            ..Default::default()
        },
    )
    .unwrap();
    let err = dam
        .edit(
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
    let s = Scenario::default();
    s.working(task(1));
    let id = oid(1);
    let dam = s.dam();
    let err = dam
        .edit(
            &id,
            EditFields {
                recurrence: Some(Some("weekly".into())),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(err, UseCaseError::Parse(_)));
    let out = dam
        .edit(
            &id,
            EditFields {
                recurrence: Some(Some("every! week".into())),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        out.base().recurrence.as_ref().map(ToString::to_string),
        Some("every! week".into())
    );
}

#[test]
fn attach_needs_an_existing_event() {
    let s = Scenario::default();
    s.working(task(1));
    let id = oid(1);
    let dam = s.dam();
    let err = dam
        .edit(
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
    s.working(event(9));
    let out = dam
        .edit(
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
    let s = Scenario::default();
    s.working(event(9));
    let dam = s.dam();
    let err = dam
        .edit(
            &oid(9),
            EditFields {
                priority: Some(Priority::HIGHEST),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NotATask(oid(9))));
}
