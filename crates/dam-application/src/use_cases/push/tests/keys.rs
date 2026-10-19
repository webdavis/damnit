use dam_domain::{Object, Task};

use crate::testing::{Scenario, accepting_all, oid, pushing, remotes_launched_by, task};

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
fn two_successive_changes_to_one_object_carry_different_keys() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "second")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2);
    assert_ne!(sent[0].idempotency_key, sent[1].idempotency_key);
}
