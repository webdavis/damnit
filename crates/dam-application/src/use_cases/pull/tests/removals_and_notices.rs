use super::cancellations::{meeting, sending_the_meeting_cancelled, tracked};
use super::incoming;
use crate::ports::{Clock, Notice};
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, todoist};
use dam_domain::{Object, Task};

#[test]
fn a_removal_upstream_is_a_notice_not_a_deletion() {
    let s = Scenario::default();
    s.committed(Object::Task(Task::new(oid(1), "keep me")))
        .tracked_as(&oid(1), "r1");
    let l = pulling(PullOutcome {
        removed: vec!["r1".into()],
        ..PullOutcome::default()
    });
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].removed_upstream, 1);
    assert!(s.store.get(&oid(1)).unwrap().is_some());
    assert!(matches!(
        s.store.notices().unwrap()[0],
        Notice::RemovedUpstream { .. }
    ));
    assert!(
        s.store
            .oid_for_remote_id(&todoist(), "r1")
            .unwrap()
            .is_none()
    );
    assert!(
        s.store
            .remote_snapshot(&todoist(), &oid(1))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_cancelled_event_with_attached_tasks_is_reported() {
    let s = tracked(meeting());
    let mut attached = Task::new(oid(6), "prep");
    attached.event = Some(oid(5));
    s.working(Object::Task(attached));
    let l = sending_the_meeting_cancelled();
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert!(matches!(
        s.store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
    assert!(s.store.get(&oid(6)).unwrap().is_some());
}

#[test]
fn an_unrelated_staged_object_stays_staged_after_a_pull() {
    let s = Scenario::default();
    s.committed(Object::Task(Task::new(oid(1), "base")))
        .tracked_as(&oid(1), "r1")
        .working(Object::Task(Task::new(oid(2), "unrelated")));
    s.dam().add_all().unwrap();
    let l = pulling(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store
            .staged()
            .unwrap()
            .iter()
            .map(|c| c.oid().clone())
            .collect::<Vec<_>>(),
        vec![oid(2)]
    );
}

#[test]
fn a_pull_records_when_it_happened() {
    let s = Scenario::default();
    let l = pulling(PullOutcome::default());
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store.last_pull(&todoist()).unwrap(),
        Some(s.fixture.clock.now())
    );
}
