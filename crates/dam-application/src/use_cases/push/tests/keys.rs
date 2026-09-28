use jiff::civil::date;

use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::ports::Repositories;
use crate::remote::MutationOutcome;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Object, Task};

#[test]
fn a_resend_after_an_unanswered_push_carries_the_first_attempt_s_key() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, pushed) = launcher(|_| vec![]);
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
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
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: Some("r1".into()),
                why: None,
            })
            .collect()
    });
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    store
        .put(&Object::Task(Task::new(oid(1), "second")))
        .unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 19)),
        &FixedRandom::new(9),
        "m2",
    )
    .unwrap();
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2);
    assert_ne!(sent[0].idempotency_key, sent[1].idempotency_key);
}
