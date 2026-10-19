use std::cell::RefCell;
use std::rc::Rc;

use super::super::*;
use super::{clock, committed_task, config, launcher};
use crate::errors::Refusal;
use crate::ports::{Conflict, HelperError};
use crate::remote::PullOutcome;
use crate::testing::prelude::*;
use crate::testing::{
    FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, oid, remotes_launched_by, task_caps,
};
use dam_domain::{Object, Task};

#[test]
fn unresolved_conflicts_block_push() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    store
        .mark_conflict(&Conflict {
            remote: RemoteName("todoist".into()),
            oid: oid(1),
            ours: Object::Task(Task::new(oid(1), "t1")),
            theirs: Object::Task(Task::new(oid(1), "theirs")),
        })
        .unwrap();
    let (l, _) = launcher(|_| vec![]);
    let err = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::UnresolvedConflicts(1)));
}

#[test]
fn an_unknown_remote_is_refused() {
    let store = MemoryStore::new();
    let (l, _) = launcher(|_| vec![]);
    let err = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), Some("nope"))
    .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
    );
}

#[test]
fn a_credential_the_config_does_not_supply_is_refused_by_name() {
    let store = MemoryStore::new();
    let mut config = config();
    config.remotes[0].credentials.clear();
    let (l, _) = launcher(|_| vec![]);
    let err = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config,
    }
    .push(remotes_launched_by(&l), None)
    .unwrap_err();
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

#[test]
fn a_helper_that_times_out_leaves_the_store_as_it_was() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    store
        .set_accepted_kinds(&RemoteName("todoist".into()), &task_caps().kinds)
        .unwrap();
    let before = store.clone();
    let timeout = HelperError::Timeout {
        helper: "todoist".into(),
        deadline: "30s".into(),
        said: None,
    };
    let answer = timeout.clone();
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: Ok(PullOutcome::default()),
            push_answer: Box::new({
                let answer = answer.clone();
                move |_| Err(answer.clone())
            }),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    let err = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
    .unwrap_err();
    assert_eq!(err, UseCaseError::Helper(timeout));
    assert_eq!(store, before);
}
