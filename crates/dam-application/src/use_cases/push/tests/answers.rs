//! What dam makes of the helper's answers: a reported failure, no answer
//! at all, and answers about mutations dam did not send.

use super::super::*;
use super::{committed_task, config, launcher};
use crate::ports::Repositories;
use crate::remote::MutationOutcome;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, MemoryStore, oid};

#[test]
fn a_failed_mutation_becomes_a_notice_and_a_retry() {
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
    let reports = push(repos, &l, &EchoCredentials, &config(), None).unwrap();
    assert_eq!(
        reports[0].failed,
        vec![(oid(1), "rate limited".to_string())]
    );
    assert_eq!(
        store.push_retries(&RemoteName("todoist".into())).unwrap(),
        vec![oid(1)]
    );
    assert!(matches!(
        store.notices().unwrap()[0],
        Notice::PushFailed { .. }
    ));
    // the commit is marked pushed; the retry set carries the failure forward
    assert!(
        store
            .unpushed(&RemoteName("todoist".into()))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_mutation_the_helper_never_answered_is_a_failure() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    committed_task(&store, 2);
    let (l, _) = launcher(|ms| {
        ms.iter()
            .filter(|m| m.oid == oid(1))
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: Some("r1".into()),
                why: None,
            })
            .collect()
    });
    let reports = push(repos, &l, &EchoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    assert_eq!(
        reports[0].failed,
        vec![(oid(2), "no answer from helper".to_string())]
    );
    assert_eq!(
        store.push_retries(&RemoteName("todoist".into())).unwrap(),
        vec![oid(2)]
    );
    assert!(matches!(
        store.notices().unwrap()[0],
        Notice::PushFailed { .. }
    ));
    let still_unpushed = store.unpushed(&RemoteName("todoist".into())).unwrap();
    assert_eq!(still_unpushed.len(), 1);
    assert!(still_unpushed[0].changes.iter().any(|c| c.oid == oid(2)));
}

#[test]
fn duplicate_and_unsent_results_are_ignored() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, _) = launcher(|_| {
        vec![
            MutationOutcome {
                oid: oid(1),
                ok: false,
                remote_id: None,
                why: Some("first".into()),
            },
            MutationOutcome {
                oid: oid(1),
                ok: false,
                remote_id: None,
                why: Some("second".into()),
            },
            MutationOutcome {
                oid: oid(9),
                ok: false,
                remote_id: None,
                why: Some("unsent".into()),
            },
        ]
    });
    let reports = push(repos, &l, &EchoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 0);
    assert_eq!(reports[0].failed.len(), 1);
    let retries = store.push_retries(&RemoteName("todoist".into())).unwrap();
    assert_eq!(retries, vec![oid(1)]);
    assert!(!retries.contains(&oid(9)));
}
