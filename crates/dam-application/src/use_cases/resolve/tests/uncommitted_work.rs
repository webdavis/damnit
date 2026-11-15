use super::super::*;
use crate::remote::{IncomingObject, MutationOp, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{Scenario, accepting_all, oid, pulling, pushing, remotes_launched_by};
use crate::use_cases::prefix::Named;
use dam_domain::{Object, Task};

fn subject(text: &str) -> Object {
    Object::Task(Task::new(oid(1), text))
}

fn removed_here_while_the_app_renamed_it() -> Scenario {
    let s = Scenario::default();
    s.committed(subject("base"))
        .tracked_as(&oid(1), "r1")
        .pushed();
    s.store.delete(&oid(1)).unwrap();
    s.dam().add(&[Named::Object(oid(1))]).unwrap();
    let theirs = IncomingObject {
        remote_id: "r1".into(),
        object: subject("theirs"),
    };
    let l = pulling(PullOutcome {
        objects: vec![theirs],
        ..PullOutcome::default()
    });
    let reports = s.dam().pull(remotes_launched_by(&l), None).unwrap();
    assert_eq!(reports[0].conflicts, 1);
    s
}

fn sent_by_the_next_push(s: &Scenario) -> Vec<MutationOp> {
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed.borrow().iter().map(|m| m.op).collect()
}

#[test]
fn ours_keeps_a_delete_made_here_and_the_next_push_sends_it() {
    let s = removed_here_while_the_app_renamed_it();
    s.dam().resolve(&oid(1), Side::Ours).unwrap();
    assert!(s.store.conflicts().unwrap().is_empty());
    assert_eq!(s.store.get(&oid(1)).unwrap(), None);
    assert_eq!(s.store.committed(&oid(1)).unwrap(), None);
    assert!(s.store.staged().unwrap().is_empty());
    assert_eq!(sent_by_the_next_push(&s), vec![MutationOp::Delete]);
}

#[test]
fn theirs_brings_back_what_the_app_holds_and_drops_the_staged_delete() {
    let s = removed_here_while_the_app_renamed_it();
    s.dam().resolve(&oid(1), Side::Theirs).unwrap();
    assert!(s.store.conflicts().unwrap().is_empty());
    assert_eq!(s.store.get(&oid(1)).unwrap(), Some(subject("theirs")));
    assert_eq!(s.store.committed(&oid(1)).unwrap(), Some(subject("theirs")));
    assert!(s.store.staged().unwrap().is_empty());
    assert!(!sent_by_the_next_push(&s).contains(&MutationOp::Delete));
}
