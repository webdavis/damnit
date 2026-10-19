use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::*;
use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::ports::{HelperError, Notice, StoreError};
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    FaultyStore, FixedClock, FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, oid,
    remotes_launched_by, task_caps,
};
use dam_domain::{Object, Task};

const MINUTE: Duration = Duration::from_secs(60);

fn remote() -> RemoteName {
    RemoteName("t".into())
}

fn config(stale: Option<Duration>) -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: remote(),
            helper: "t".into(),
            url: "t::".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: "t".into(),
            }],
            stale,
            deadline: None,
            path: None,
        }],
        ..Config::default()
    }
}

fn answering(answer: Result<PullOutcome, HelperError>) -> ScriptedLauncher {
    ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: answer.clone(),
            push_answer: Box::new(|_| Ok(vec![])),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    }
}

fn one_object() -> Result<PullOutcome, HelperError> {
    Ok(PullOutcome {
        objects: vec![IncomingObject {
            remote_id: "r1".into(),
            object: Object::Task(Task::new(oid(0), "from upstream")),
        }],
        ..PullOutcome::default()
    })
}

fn clock() -> FixedClock {
    FixedClock(date(2026, 9, 18))
}

fn refresh(
    store: &MemoryStore,
    launcher: &ScriptedLauncher,
    config: &Config,
) -> Result<(), UseCaseError> {
    Dam {
        store,
        clock: &clock(),
        random: &FixedRandom::new(7),
        config,
    }
    .refresh_stale(remotes_launched_by(launcher))
}

fn launches(launcher: &ScriptedLauncher) -> usize {
    launcher.launched_with.borrow().len()
}

#[test]
fn a_stale_remote_is_pulled_and_a_fresh_one_is_not() {
    let store = MemoryStore::new();
    let launcher = answering(one_object());
    refresh(&store, &launcher, &config(Some(MINUTE))).unwrap();
    assert_eq!(store.all().unwrap().len(), 1);
    refresh(&store, &launcher, &config(Some(MINUTE))).unwrap();
    assert_eq!(launches(&launcher), 1, "a fresh remote is not pulled again");
}

#[test]
fn a_remote_without_a_stale_window_is_never_pulled() {
    let store = MemoryStore::new();
    let launcher = answering(one_object());
    refresh(&store, &launcher, &config(None)).unwrap();
    assert_eq!(launches(&launcher), 0);
}

#[test]
fn an_unreachable_stale_remote_leaves_a_notice_and_no_last_pull() {
    let store = MemoryStore::new();
    let launcher = answering(Err(HelperError::Io("down".into())));
    refresh(&store, &launcher, &config(Some(MINUTE))).unwrap();
    assert_eq!(store.last_pull(&remote()).unwrap(), None);
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::PullFailed { remote: r, .. }) if *r == remote()
    ));
}

fn pulled_ago(ago: Duration) -> MemoryStore {
    let store = MemoryStore::new();
    let at = clock()
        .now()
        .checked_sub(jiff::SignedDuration::try_from(ago).unwrap())
        .unwrap();
    store.set_last_pull(&remote(), at).unwrap();
    store
}

#[test]
fn a_remote_pulled_exactly_one_window_ago_is_fresh_and_one_second_later_stale() {
    let at_the_window = pulled_ago(MINUTE);
    let launcher = answering(one_object());
    refresh(&at_the_window, &launcher, &config(Some(MINUTE))).unwrap();
    assert_eq!(
        launches(&launcher),
        0,
        "a pull exactly one window old is fresh"
    );
    let past_it = pulled_ago(MINUTE + Duration::from_secs(1));
    refresh(&past_it, &launcher, &config(Some(MINUTE))).unwrap();
    assert_eq!(
        launches(&launcher),
        1,
        "a pull older than the window is stale"
    );
}

#[test]
fn a_store_error_reading_the_last_pull_fails_the_read_and_pulls_nothing() {
    let store = FaultyStore::new(MemoryStore::new());
    store.fail_reading_the_last_pull();
    let launcher = answering(one_object());
    let refused = Dam {
        store: &store,
        clock: &clock(),
        random: &FixedRandom::new(7),
        config: &config(Some(MINUTE)),
    }
    .refresh_stale(remotes_launched_by(&launcher));
    assert_eq!(
        refused,
        Err(UseCaseError::Store(StoreError::Failed(
            "the last pull could not be read".into()
        )))
    );
    assert_eq!(launches(&launcher), 0);
}
