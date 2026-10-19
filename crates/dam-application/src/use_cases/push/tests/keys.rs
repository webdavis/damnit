use jiff::civil::date;

use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::remote::{MutationOutcome, MutationVerdict};
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, Fixture, MemoryStore, oid, remotes_launched_by};
use dam_domain::{Object, Task};

#[test]
fn a_resend_after_an_unanswered_push_carries_the_first_attempt_s_key() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, pushed) = launcher(|_| vec![]);
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
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
    committed_task(&store, 1);
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Accepted {
                    remote_id: Some("r1".into()),
                },
            })
            .collect()
    });
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    store
        .put(&Object::Task(Task::new(oid(1), "second")))
        .unwrap();
    let fixture = Fixture {
        clock: FixedClock(date(2026, 9, 19)),
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.commit("m2").unwrap();
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    let sent = pushed.borrow();
    assert_eq!(sent.len(), 2);
    assert_ne!(sent[0].idempotency_key, sent[1].idempotency_key);
}
