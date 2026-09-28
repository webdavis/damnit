mod answers;
mod keys;
mod last_push;
mod refusals;
mod verbs;

use std::cell::RefCell;
use std::rc::Rc;

use crate::remote::{MutationOp, MutationOutcome, PullOutcome, RemoteMutation};
use dam_domain::{Event, Object, Task, When};
use jiff::civil::date;

use super::*;
use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::ports::RemoteName;
use crate::ports::Repositories;
use crate::testing::prelude::*;
use crate::testing::{
    EchoCredentials, FixedClock, FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;

pub(super) fn config() -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: RemoteName("todoist".into()),
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

pub(super) fn launcher(
    answer: impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static,
) -> (ScriptedLauncher, Rc<RefCell<Vec<RemoteMutation>>>) {
    let pushed = Rc::new(RefCell::new(Vec::new()));
    let seen = pushed.clone();
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: PullOutcome::default(),
            push_answer: Box::new(answer.clone()),
            pushed: seen.clone(),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    (l, pushed)
}

pub(super) fn clock() -> FixedClock {
    FixedClock(date(2026, 9, 18))
}

pub(super) fn committed_task(store: &MemoryStore, byte: u8) {
    store
        .put(&Object::Task(Task::new(oid(byte), format!("t{byte}"))))
        .unwrap();
    add_all(store, store, store).unwrap();
    commit(
        store,
        store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(byte),
        "m",
    )
    .unwrap();
}

#[test]
fn a_committed_create_is_sent_and_its_remote_id_is_mapped() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: Some("r1".into()),
                why: None,
            })
            .collect()
    });
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    assert_eq!(pushed.borrow()[0].op, MutationOp::Create);
    assert_eq!(
        store
            .remote_id(&RemoteName("todoist".into()), &oid(1))
            .unwrap()
            .as_deref(),
        Some("r1")
    );
    assert!(
        store
            .unpushed(&RemoteName("todoist".into()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        *l.launched_with.borrow(),
        vec![vec![("api_token".to_string(), "t".into())]],
        "one helper process, launched with the token the remote's config declares"
    );
}

#[test]
fn a_retry_is_sent_as_an_update_of_the_committed_state() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    committed_task(&store, 1);
    store
        .map_remote_id(&RemoteName("todoist".into()), &oid(1), "r1")
        .unwrap();
    store
        .set_push_retries(&RemoteName("todoist".into()), &[oid(1)])
        .unwrap();
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: None,
                why: None,
            })
            .collect()
    });
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    let sent = pushed.borrow();
    assert!(
        sent.iter()
            .any(|m| m.op == MutationOp::Update && m.remote_id.as_deref() == Some("r1"))
    );
    assert!(
        store
            .push_retries(&RemoteName("todoist".into()))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_successful_delete_clears_the_remote_mapping() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let remote = RemoteName("todoist".into());
    store.put(&Object::Task(Task::new(oid(1), "t1"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    let created = commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote, &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote, &Object::Task(Task::new(oid(1), "t1")))
        .unwrap();
    store.mark_pushed(&remote, &created.id).unwrap();

    store.delete(&oid(1)).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(2),
        "rm",
    )
    .unwrap();

    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: None,
                why: None,
            })
            .collect()
    });
    push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert!(store.remote_id(&remote, &oid(1)).unwrap().is_none());
    assert!(store.remote_snapshot(&remote, &oid(1)).unwrap().is_none());
}

#[test]
fn a_delete_with_no_remote_mapping_is_skipped() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "t1"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    let created = commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        "m",
    )
    .unwrap();
    store
        .mark_pushed(&RemoteName("todoist".into()), &created.id)
        .unwrap();

    store.delete(&oid(1)).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(2),
        "rm",
    )
    .unwrap();

    let (l, pushed) = launcher(|_| vec![]);
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert_eq!(reports[0].sent, 0);
    assert!(pushed.borrow().is_empty());
}

#[test]
fn an_event_is_skipped_by_a_task_only_helper() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let e = Event::new(
        oid(2),
        "e",
        When::Day(date(2026, 1, 1)),
        When::Day(date(2026, 1, 2)),
    );
    store.put(&Object::Event(e)).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(3),
        "m",
    )
    .unwrap();
    let (l, pushed) = launcher(|_| vec![]);
    let reports = push(repos, &l, &EchoCredentials, &clock(), &config(), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert!(pushed.borrow().is_empty());
}
