//! What a push refuses before it sends anything.

use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::errors::Refusal;
use crate::ports::Repositories;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, MemoryStore, oid};
use dam_domain::{Object, Task};

#[test]
fn unresolved_conflicts_block_push() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    store
        .mark_conflict(
            &RemoteName("todoist".into()),
            &oid(1),
            &Object::Task(Task::new(oid(1), "theirs")),
        )
        .unwrap();
    let (l, _) = launcher(|_| vec![]);
    let err = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::UnresolvedConflicts(1)));
}

#[test]
fn an_unknown_remote_is_refused() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let (l, _) = launcher(|_| vec![]);
    let err = push(
        repos,
        &l,
        &EchoCredentials,
        &clock(),
        &config(),
        Some("nope"),
    )
    .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
    );
}

/// The helper stays the authority on what it needs, even though the config
/// is what supplies it: a name the helper declares and the config does not
/// is refused by name rather than sent as nothing.
#[test]
fn a_credential_the_config_does_not_supply_is_refused_by_name() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let mut config = config();
    config.remotes[0].credentials.clear();
    let (l, _) = launcher(|_| vec![]);
    let err = push(repos, &l, &EchoCredentials, &clock(), &config, None).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::MissingCredential {
            remote: "todoist".into(),
            name: "api_token".into()
        })
    );
    assert_eq!(
        l.launched_with.borrow().len(),
        1,
        "one process, even when the capabilities it reported cannot be met"
    );
}
