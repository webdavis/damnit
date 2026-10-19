use std::cell::RefCell;
use std::rc::Rc;

use crate::ports::RemoteName;
use crate::remote::{PullOutcome, RemoteCapabilities, RemoteMutation};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{
    FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, remotes_launched_by, task_caps,
};

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
            pull_answer: Ok(PullOutcome::default()),
            push_answer: Box::new(|_: &[RemoteMutation]| {
                panic!("a remote that declares no kinds was sent a push")
            }),
            pushed: seen.clone(),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    let reports = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(0),
        config: &config(),
    }
    .push(remotes_launched_by(&l), None)
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
