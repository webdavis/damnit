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
    assert_eq!(pulled.before(), Some(&old));
}
