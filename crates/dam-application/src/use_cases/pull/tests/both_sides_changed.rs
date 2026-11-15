use super::incoming;
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pulling, remotes_launched_by};
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
