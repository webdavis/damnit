//! A remote that declares no kinds is read-only: a push sends it nothing.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ports::{RemoteName, Repositories};
use crate::remote::{MutationOutcome, PullOutcome, RemoteCapabilities, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, MemoryStore, ScriptedHelper, ScriptedLauncher, task_caps};
use crate::use_cases::push::push;

use super::{clock, committed_task, config};

#[test]
fn a_remote_that_declares_no_kinds_is_sent_nothing_and_owed_nothing() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let pushed = Rc::new(RefCell::new(Vec::new()));
    let seen = pushed.clone();
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: RemoteCapabilities {
                kinds: vec![],
                ..task_caps()
            },
            pull_answer: PullOutcome::default(),
            push_answer: Box::new(|_: &[RemoteMutation]| -> Vec<MutationOutcome> {
                panic!("a remote that declares no kinds was sent a push")
            }),
            pushed: seen.clone(),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    let reports = push(
        Repositories::of(&store),
        &l,
        &EchoCredentials,
        &clock(),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!((reports[0].sent, reports[0].skipped), (0, 1));
    assert!(pushed.borrow().is_empty());
    assert!(
        store
            .unpushed(&RemoteName("todoist".into()))
            .unwrap()
            .is_empty(),
        "nothing stays owed to it"
    );
    assert!(
        store.notices().unwrap().is_empty(),
        "nothing was refused, so nothing is reported"
    );
}
