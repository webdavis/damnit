use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::ports::Clock;
use crate::remote::{MutationOutcome, MutationVerdict};
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, FixedRandom, MemoryStore, remotes_launched_by, task_caps};

#[test]
fn a_push_that_reached_the_remote_records_when_it_did() {
    let store = MemoryStore::new();
    let todoist = RemoteName("todoist".into());
    assert_eq!(store.last_push(&todoist).unwrap(), None);
    committed_task(&store, 1);
    let (l, _) = launcher(|ms| {
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
    assert_eq!(store.last_push(&todoist).unwrap(), Some(clock().now()));
}

#[test]
fn a_push_with_nothing_to_send_records_the_time_too() {
    let store = MemoryStore::new();
    let (l, pushed) = launcher(|_| vec![]);
    Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap();
    assert!(pushed.borrow().is_empty());
    assert_eq!(
        store.last_push(&RemoteName("todoist".into())).unwrap(),
        Some(clock().now())
    );
}

#[test]
fn a_push_whose_mutations_the_remote_refused_still_records_when_it_did() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Refused {
                    why: "rate limited".into(),
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
    assert_eq!(
        store.last_push(&RemoteName("todoist".into())).unwrap(),
        Some(clock().now())
    );
}

#[test]
fn push_bookkeeping_that_fails_midway_records_nothing() {
    let store = FaultyStore::new(MemoryStore::new());
    let todoist = RemoteName("todoist".into());
    committed_task(&store.inner, 1);
    store
        .inner
        .set_accepted_kinds(&todoist, &task_caps().kinds)
        .unwrap();
    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Accepted {
                    remote_id: Some("r1".into()),
                },
            })
            .collect()
    });
    store.assert_unchanged_when_any_write_fails(|| {
        Dam {
            store: &store,
            clock: &clock(),
            random: &FixedRandom::new(0),
            config: &config(),
        }
        .push(remotes_launched_by(&l), None)
        .map(drop)
    });
}
