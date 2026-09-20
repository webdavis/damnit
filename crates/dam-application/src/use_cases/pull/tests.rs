use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::*;
use crate::config::{CredentialSpec, RemoteConfig};
use crate::ports::Repositories;
use crate::testing::prelude::*;
use crate::testing::{
    FixedClock, FixedRandom, MemoryStore, NoCredentials, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use crate::wire::to_wire;
use dam_domain::{Object, Task};
use dam_protocol::{PullResponse, WireObject};

/// Shared with `tests_removals_and_notices`, split out by concern to keep
/// each file under the size cap.
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

pub(super) fn launcher(answer: PullResponse) -> ScriptedLauncher {
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

pub(super) fn wire(subject: &str, remote_id: &str) -> WireObject {
    let mut w = to_wire(
        &Object::Task(Task::new(oid(0), subject)),
        Some(remote_id.into()),
    );
    w.oid = String::new();
    w
}

#[test]
fn a_new_upstream_object_is_created_locally_with_a_fresh_oid_and_mapped() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let l = launcher(PullResponse {
        objects: vec![wire("from todoist", "r1")],
        removed: vec![],
        sync: Some("s1".into()),
    });
    let reports = pull(
        repos,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
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
        &mut FixedRandom(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "old")))
        .unwrap();
    let l = launcher(PullResponse {
        objects: vec![wire("new", "r1")],
        removed: vec![],
        sync: None,
    });
    let reports = pull(
        repos,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
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
        &mut FixedRandom(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(t))
        .unwrap();
    let l = launcher(PullResponse {
        objects: vec![wire("new", "r1")],
        removed: vec![],
        sync: None,
    });
    pull(
        repos,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
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
fn both_sides_changed_is_a_conflict_and_nothing_is_overwritten() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    // local moves and is committed
    let mut mine = store.get(&oid(1)).unwrap().unwrap();
    mine.base_mut().subject = "mine".into();
    store.put(&mine).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(10),
        "m2",
    )
    .unwrap();
    let l = launcher(PullResponse {
        objects: vec![wire("theirs", "r1")],
        removed: vec![],
        sync: None,
    });
    let reports = pull(
        repos,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].conflicts, 1);
    assert_eq!(store.get(&oid(1)).unwrap().unwrap().base().subject, "mine");
    let c = &store.conflicts().unwrap()[0];
    assert_eq!(c.theirs.base().subject, "theirs");
}

#[test]
fn uncommitted_local_work_stops_the_pull_before_anything_is_written() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(9),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    let mut dirty = store.get(&oid(1)).unwrap().unwrap();
    dirty.base_mut().subject = "unsaved".into();
    store.put(&dirty).unwrap();
    let l = launcher(PullResponse {
        objects: vec![wire("theirs", "r1"), wire("brand new", "r2")],
        removed: vec![],
        sync: Some("s9".into()),
    });
    let err = pull(
        repos,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::DirtyOnPull { oid: oid(1) })
    );
    assert!(store.oid_for_remote_id(&remote(), "r2").unwrap().is_none());
    assert!(store.sync_token(&remote()).unwrap().is_none());
}
