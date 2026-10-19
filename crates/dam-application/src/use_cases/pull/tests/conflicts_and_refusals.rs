use super::super::*;
use super::incoming;
use crate::errors::Refusal;
use crate::remote::{PullOutcome, RejectedObject};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, todoist};
use dam_domain::{Object, Task};

fn subject(text: &str) -> Object {
    Object::Task(Task::new(oid(1), text))
}

fn tracked_base() -> Scenario {
    let s = Scenario::default();
    s.committed(subject("base")).tracked_as(&oid(1), "r1");
    s
}

#[test]
fn both_sides_changed_is_a_conflict_and_nothing_is_overwritten() {
    let s = tracked_base();
    s.committed(subject("mine"));
    let l = pulling(PullOutcome {
        objects: vec![incoming("theirs", "r1")],
        ..PullOutcome::default()
    });
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].conflicts, 1);
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "mine"
    );
    let c = &s.store.conflicts().unwrap()[0];
    assert_eq!(c.theirs.base().subject, "theirs");
}

#[test]
fn uncommitted_local_work_stops_the_pull_before_anything_is_written() {
    let s = tracked_base();
    s.working(subject("unsaved"));
    let l = pulling(PullOutcome {
        objects: vec![incoming("theirs", "r1"), incoming("brand new", "r2")],
        sync: Some("s9".into()),
        ..PullOutcome::default()
    });
    let err = s.dam().pull(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::DirtyOnPull { oid: oid(1) })
    );
    assert!(
        s.store
            .oid_for_remote_id(&todoist(), "r2")
            .unwrap()
            .is_none()
    );
    assert!(s.store.sync_token(&todoist()).unwrap().is_none());
}

#[test]
fn a_refused_pull_leaves_none_of_its_own_notices_behind() {
    let s = tracked_base();
    s.working(subject("unsaved"));
    let l = pulling(PullOutcome {
        objects: vec![incoming("theirs", "r1")],
        rejected: vec![RejectedObject {
            remote_id: "r-bad".into(),
            why: "path: cannot read \"a//b\"".into(),
        }],
        sync: Some("s9".into()),
        ..PullOutcome::default()
    });
    let err = s.dam().pull(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::DirtyOnPull { oid: oid(1) })
    );
    assert!(
        s.store.notices().unwrap().is_empty(),
        "the rejection notice was written before the refusal and rolled back with it"
    );
}
