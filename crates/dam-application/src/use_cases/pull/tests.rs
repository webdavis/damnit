mod accepted_kinds;
mod cancellations;
mod cancelled_conflicts;
mod conflicts_and_refusals;
mod helper_failures;
mod history;
mod kind_change;
mod removals_and_notices;
mod skipped_objects;
mod skipped_twice;

use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by, todoist};
use dam_domain::{Object, Task};

pub(super) fn incoming(subject: &str, remote_id: &str) -> IncomingObject {
    IncomingObject {
        remote_id: remote_id.to_string(),
        object: Object::Task(Task::new(oid(0), subject)),
    }
}

#[test]
fn a_new_upstream_object_is_created_locally_with_a_fresh_oid_and_mapped() {
    let s = Scenario::default().drawing_from(42);
    let l = pulling(PullOutcome {
        objects: vec![incoming("from todoist", "r1")],
        sync: Some("s1".into()),
        ..PullOutcome::default()
    });
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].created, 1);
    let local = s.store.get(&oid(42)).unwrap().unwrap();
    assert_eq!(local.base().subject, "from todoist");
    assert_eq!(s.store.committed(&oid(42)).unwrap(), Some(local));
    assert_eq!(
        s.store.oid_for_remote_id(&todoist(), "r1").unwrap(),
        Some(oid(42))
    );
    assert_eq!(
        s.store.sync_token(&todoist()).unwrap().as_deref(),
        Some("s1")
    );
}

#[test]
fn a_fast_forward_updates_a_clean_local_object() {
    let s = Scenario::default();
    s.committed(Object::Task(Task::new(oid(1), "old")))
        .tracked_as(&oid(1), "r1");
    let l = pulling(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].updated, 1);
    assert_eq!(s.store.get(&oid(1)).unwrap().unwrap().base().subject, "new");
    assert_eq!(
        s.store.committed(&oid(1)).unwrap().unwrap().base().subject,
        "new"
    );
}

#[test]
fn dam_only_fields_survive_a_pull() {
    let s = Scenario::default();
    let mut t = Task::new(oid(1), "old");
    t.base.depends.push(oid(7));
    s.committed(Object::Task(t)).tracked_as(&oid(1), "r1");
    let l = pulling(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().depends,
        vec![oid(7)]
    );
}

#[test]
fn a_pull_commit_is_already_pushed_to_the_remote_it_came_from() {
    let s = Scenario::default();
    let l = pulling(PullOutcome {
        objects: vec![incoming("from todoist", "r1")],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(s.store.log().unwrap().len(), 1);
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}
