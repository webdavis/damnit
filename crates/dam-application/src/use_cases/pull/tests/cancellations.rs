use dam_domain::{Event, EventStatus, Field, Object, Task, When};
use jiff::civil::date;

use super::super::*;
use crate::errors::Refusal;
use crate::ports::Notice;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, ScriptedLauncher, oid, pulling_events, remotes_launched_by, todoist,
};

pub(super) fn meeting() -> Event {
    Event::new(
        oid(5),
        "meeting",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    )
}

pub(super) fn tracked(event: Event) -> Scenario {
    let s = Scenario::default();
    s.committed(Object::Event(event)).tracked_as(&oid(5), "e1");
    s
}

pub(super) fn every_event_field() -> Vec<Field> {
    vec![Field::Subject, Field::Start, Field::End, Field::Status]
}

pub(super) fn sending_the_meeting_cancelled() -> ScriptedLauncher {
    let mut cancelled = meeting();
    cancelled.status = EventStatus::Cancelled;
    pulling_events(
        every_event_field(),
        PullOutcome {
            objects: vec![IncomingObject {
                remote_id: "e1".into(),
                object: Object::Event(cancelled),
            }],
            ..PullOutcome::default()
        },
    )
}

fn declaring(ids: &[&str], fields: Vec<Field>) -> ScriptedLauncher {
    pulling_events(
        fields,
        PullOutcome {
            cancelled: ids.iter().map(|s| s.to_string()).collect(),
            ..PullOutcome::default()
        },
    )
}

pub(super) fn run(s: &Scenario, ids: &[&str]) -> Result<Vec<PullReport>, UseCaseError> {
    let l = declaring(ids, every_event_field());
    s.dam().pull(remotes_launched_by(&l), None)
}

#[test]
fn a_cancelled_id_moves_the_event_it_maps_to_cancelled() {
    let s = tracked(meeting());
    let reports = run(&s, &["e1"]).unwrap();
    assert_eq!(reports[0].updated, 1);
    let Some(Object::Event(now)) = s.store.get(&oid(5)).unwrap() else {
        panic!("the event is gone")
    };
    assert_eq!(now.status, EventStatus::Cancelled);
    assert_eq!(now.base.subject, "meeting", "only the status moved");
    assert_eq!(
        s.store.oid_for_remote_id(&todoist(), "e1").unwrap(),
        Some(oid(5)),
        "still tracked"
    );
}

#[test]
fn a_helper_that_does_not_declare_status_cancels_nothing() {
    let s = tracked(meeting());
    let silent = declaring(&["e1"], vec![Field::Subject, Field::Start, Field::End]);
    let reports = s.dam().pull(remotes_launched_by(&silent), None).unwrap();
    assert_eq!((reports[0].updated, reports[0].unchanged), (0, 1));
    let Some(Object::Event(now)) = s.store.get(&oid(5)).unwrap() else {
        panic!("the event is gone")
    };
    assert_eq!(now.status, EventStatus::Confirmed);
}

#[test]
fn a_cancelled_event_with_an_attached_task_is_reported_and_the_task_kept() {
    let s = tracked(meeting());
    let mut prep = Task::new(oid(6), "prep");
    prep.event = Some(oid(5));
    s.working(Object::Task(prep));
    run(&s, &["e1"]).unwrap();
    assert!(matches!(
        s.store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
    assert!(s.store.get(&oid(6)).unwrap().is_some());
}

#[test]
fn an_id_dam_never_mapped_and_one_mapped_to_a_task_change_nothing() {
    let s = Scenario::default();
    s.working(Object::Task(Task::new(oid(7), "a task")))
        .mapped_as(&oid(7), "t1");
    let reports = run(&s, &["never-seen", "t1"]).unwrap();
    let r = &reports[0];
    assert_eq!(
        (r.created, r.updated, r.unchanged, r.conflicts),
        (0, 0, 0, 0),
        "{r:?}"
    );
    assert!(matches!(
        s.store.get(&oid(7)).unwrap(),
        Some(Object::Task(_))
    ));
}

#[test]
fn an_event_already_cancelled_is_unchanged() {
    let mut gone = meeting();
    gone.status = EventStatus::Cancelled;
    let s = tracked(gone);
    assert_eq!(run(&s, &["e1"]).unwrap()[0].unchanged, 1);
}

#[test]
fn an_uncommitted_local_edit_stops_the_pull_as_any_upstream_change_would() {
    let s = tracked(meeting());
    let mut edited = meeting();
    edited.base.subject = "renamed here".into();
    s.working(Object::Event(edited));
    assert!(matches!(
        run(&s, &["e1"]).unwrap_err(),
        UseCaseError::Refused(Refusal::DirtyOnPull { .. })
    ));
}

#[test]
fn a_committed_local_edit_and_a_cancellation_conflict_over_what_the_remote_held() {
    let s = tracked(meeting());
    let mut edited = meeting();
    edited.base.subject = "renamed here".into();
    s.committed(Object::Event(edited));
    assert_eq!(run(&s, &["e1"]).unwrap()[0].conflicts, 1);
    let Object::Event(theirs) = &s.store.conflicts().unwrap()[0].theirs else {
        panic!("not an event")
    };
    assert_eq!(
        (theirs.base.subject.as_str(), theirs.status),
        ("meeting", EventStatus::Cancelled)
    );
}
