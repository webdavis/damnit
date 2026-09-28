use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::ports::Repositories;
use crate::remote::MutationOutcome;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, MemoryStore};

#[test]
fn a_push_that_reached_the_remote_records_when_it_did() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let todoist = RemoteName("todoist".into());
    assert_eq!(store.last_push(&todoist).unwrap(), None);
    committed_task(&store, 1);
    let (l, _) = launcher(|ms| {
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
    assert_eq!(store.last_push(&todoist).unwrap(), Some(clock().now()));
}

#[test]
fn a_push_with_nothing_to_send_records_the_time_too() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let (l, pushed) = launcher(|_| vec![]);
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert!(pushed.borrow().is_empty());
    assert_eq!(
        store.last_push(&RemoteName("todoist".into())).unwrap(),
        Some(clock().now())
    );
}

#[test]
fn a_push_whose_mutations_the_remote_refused_still_records_when_it_did() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: false,
                remote_id: None,
                why: Some("rate limited".into()),
            })
            .collect()
    });
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(
        store.last_push(&RemoteName("todoist".into())).unwrap(),
        Some(clock().now())
    );
}
