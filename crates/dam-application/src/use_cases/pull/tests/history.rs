use super::incoming;
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by};
use dam_domain::{Object, Task};

#[test]
fn a_pulled_update_records_the_object_it_replaced() {
    let s = Scenario::default();
    let old = Object::Task(Task::new(oid(1), "old"));
    s.committed(old.clone()).tracked_as(&oid(1), "r1");
    let l = pulling(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    let pulled = &s.store.log().unwrap()[0].changes[0];
    assert_eq!(pulled.change.before(), Some(&old));
}

#[test]
fn a_pull_records_its_commit_as_pulled_and_an_owners_commit_is_not() {
    let s = Scenario::default();
    s.committed(Object::Task(Task::new(oid(1), "old")))
        .tracked_as(&oid(1), "r1");
    let l = pulling(PullOutcome {
        objects: vec![incoming("new", "r1")],
        ..PullOutcome::default()
    });
    s.dam().pull(remotes_launched_by(&l), None).unwrap();
    let log = s.store.log().unwrap();
    assert!(log[0].pulled, "the pull's commit");
    assert!(!log[1].pulled, "the owner's commit");
}
