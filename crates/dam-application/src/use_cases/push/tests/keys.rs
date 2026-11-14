use dam_domain::{Object, Task};

use crate::remote::{MutationOp, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, oid, pushing, refusing_all, remotes_launched_by, task, todoist,
};

fn renamed(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

fn keys_of(sent: &[RemoteMutation]) -> Vec<(MutationOp, &str)> {
    sent.iter()
        .map(|m| (m.op, m.idempotency_key.as_str()))
        .collect()
}

#[test]
fn a_resend_after_an_unanswered_push_carries_the_first_attempt_s_key() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2, "the mutation was sent twice: {sent:?}");
    assert_eq!(
        sent[0].idempotency_key, sent[1].idempotency_key,
        "the resend minted a fresh key"
    );
    assert_eq!(
        sent[0].op, sent[1].op,
        "a resend of a create is still a create"
    );
    assert!(!sent[0].idempotency_key.is_empty());
}

#[test]
fn the_same_update_resent_after_no_answer_carries_the_same_key() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(renamed("second"));
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    let keys = keys_of(&sent);
    assert_eq!(keys.len(), 2, "{sent:?}");
    assert_eq!(keys[0], keys[1]);
    assert_eq!(keys[0].0, MutationOp::Update);
}

#[test]
fn a_create_edited_before_its_resend_is_resent_under_the_first_key() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(renamed("edited"));
    let (l, resent) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let first = keys_of(&pushed.borrow())[0].1.to_string();
    let resent = resent.borrow();
    assert_eq!(
        keys_of(&resent)[0],
        (MutationOp::Create, first.as_str()),
        "{resent:?}"
    );
}

#[test]
fn two_successive_changes_to_one_object_carry_different_keys() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(renamed("second"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2);
    assert_ne!(sent[0].idempotency_key, sent[1].idempotency_key);
}

#[test]
fn two_retries_of_one_object_for_different_changes_carry_different_keys() {
    let s = Scenario::default();
    s.committed(task(1)).tracked_as(&oid(1), "r1").pushed();
    s.committed(renamed("first")).pushed();
    s.store.set_push_retries(&todoist(), &[oid(1)]).unwrap();
    let (l, pushed) = pushing(refusing_all("temporary failure"));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(renamed("second")).pushed();
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    let keys = keys_of(&sent);
    assert_eq!(keys.len(), 2, "{sent:?}");
    assert_ne!(keys[0], keys[1]);
}

#[test]
fn a_refused_create_retried_twice_carries_two_different_keys() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(refusing_all("temporary failure"));
    for _ in 0..3 {
        s.dam().push(remotes_launched_by(&l), None).unwrap();
    }
    let sent = pushed.borrow();
    let keys = keys_of(&sent);
    assert_eq!(keys.len(), 3, "{sent:?}");
    assert!(keys.iter().all(|(op, _)| *op == MutationOp::Create));
    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[1], keys[2]);
}
