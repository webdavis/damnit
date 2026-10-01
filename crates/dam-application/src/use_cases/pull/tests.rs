mod accepted_kinds;
mod cancellations;
mod conflicts_and_refusals;
mod kind_change;
mod removals_and_notices;

use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::*;
use crate::config::{CredentialSpec, RemoteConfig};
use crate::errors::Refusal;
use crate::ports::Repositories;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    EchoCredentials, FixedClock, FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Object, Task};

pub(super) fn remote() -> RemoteName {
    RemoteName("todoist".into())
}

pub(super) fn config() -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: remote(),
            helper: "todoist".into(),
            url: "todoist::".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: "t".into(),
            }],
            stale: None,
            deadline: None,
            path: None,
        }],
        ..Config::default()
    }
}

pub(super) fn launcher(answer: PullOutcome) -> ScriptedLauncher {
    ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: answer.clone(),
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    }
}

pub(super) fn incoming(subject: &str, remote_id: &str) -> IncomingObject {
    IncomingObject {
        remote_id: remote_id.to_string(),
        object: Object::Task(Task::new(oid(0), subject)),
    }
}

#[test]
fn a_new_upstream_object_is_created_locally_with_a_fresh_oid_and_mapped() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let l = launcher(PullOutcome {
        objects: vec![incoming("from todoist", "r1")],
        rejected: vec![],
        removed: vec![],
        cancelled: vec![],
        sync: Some("s1".into()),
    });
    let reports = pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].created, 1);
    let local = store.get(&oid(42)).unwrap().unwrap();
    assert_eq!(local.base().subject, "from todoist");
    assert_eq!(store.committed(&oid(42)).unwrap(), Some(local));
    assert_eq!(
        store.oid_for_remote_id(&remote(), "r1").unwrap(),
        Some(oid(42))
    );
    assert_eq!(store.sync_token(&remote()).unwrap().as_deref(), Some("s1"));
}

#[test]
fn a_fast_forward_updates_a_clean_local_object() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "old"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "old")))
        .unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("new", "r1")],
        removed: vec![],
        ..PullOutcome::default()
    });
    let reports = pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].updated, 1);
    assert_eq!(store.get(&oid(1)).unwrap().unwrap().base().subject, "new");
    assert_eq!(
        store.committed(&oid(1)).unwrap().unwrap().base().subject,
        "new"
    );
}

#[test]
fn dam_only_fields_survive_a_pull() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let mut t = Task::new(oid(1), "old");
    t.base.depends.push(oid(7));
    store.put(&Object::Task(t.clone())).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(t))
        .unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("new", "r1")],
        removed: vec![],
        ..PullOutcome::default()
    });
    pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        store.get(&oid(1)).unwrap().unwrap().base().depends,
        vec![oid(7)]
    );
}

#[test]
fn a_pull_commit_is_already_pushed_to_the_remote_it_came_from() {
    let store = MemoryStore::new();
    let l = launcher(PullOutcome {
        objects: vec![incoming("from todoist", "r1")],
        ..PullOutcome::default()
    });
    pull(
        Repositories::of(&store),
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(store.log().unwrap().len(), 1);
    assert!(store.unpushed(&remote()).unwrap().is_empty());
}
