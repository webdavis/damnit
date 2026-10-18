use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::super::*;
use super::{config, remote};
use crate::ports::{HelperError, Repositories};
use crate::testing::prelude::*;
use crate::testing::{
    EchoCredentials, FixedClock, FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher,
    task_caps,
};

#[test]
fn a_helper_that_times_out_leaves_the_store_as_it_was() {
    let store = MemoryStore::new();
    store
        .set_accepted_kinds(&remote(), &task_caps().kinds)
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
            pull_answer: Err(answer.clone()),
            push_answer: Box::new(|_| Ok(vec![])),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    let err = pull(
        Repositories::of(&store),
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        &config(),
        None,
    )
    .unwrap_err();
    assert_eq!(err, UseCaseError::Helper(timeout));
    assert_eq!(store, before);
}
