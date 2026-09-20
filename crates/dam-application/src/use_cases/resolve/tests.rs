use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::ports::RemoteName;
use crate::ports::Repositories;
use crate::remote::{IncomingObject, MutationOutcome, PullOutcome, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    EchoCredentials, FixedClock, FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::pull::pull;
use crate::use_cases::push::push;
use crate::use_cases::stage::add_all;
use dam_domain::{Object, Task};
use jiff::civil::date;

fn remote() -> RemoteName {
    RemoteName("todoist".into())
}

fn conflicted() -> MemoryStore {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    store
        .mark_conflict(
            &remote(),
            &oid(1),
            &Object::Task(Task::new(oid(1), "theirs")),
        )
        .unwrap();
    store
}

#[test]
fn theirs_takes_the_upstream_version() {
    let store = conflicted();
    resolve(
        &store,
        &store,
        &store,
        &FixedClock(jiff::civil::date(2026, 9, 18)),
        &mut FixedRandom(3),
        &oid(1),
        Side::Theirs,
    )
    .unwrap();
    assert_eq!(
        store.get(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
    assert_eq!(
        store.committed(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
    assert!(store.conflicts().unwrap().is_empty());
    let owed = store.unpushed(&remote()).unwrap();
    assert_eq!(
        owed[0].changes[0].after.as_ref().unwrap().base().subject,
        "theirs",
        "what is owed upstream ends at theirs, so a push cannot send ours back"
    );
}

#[test]
fn resolving_with_ours_leaves_an_existing_unpushed_commit_untouched() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    let created = commit(
        &store,
        &store,
        &FixedClock(jiff::civil::date(2026, 9, 18)),
        &mut FixedRandom(1),
        "m",
    )
    .unwrap();
    store
        .mark_conflict(
            &remote(),
            &oid(1),
            &Object::Task(Task::new(oid(1), "theirs")),
        )
        .unwrap();
    resolve(
        &store,
        &store,
        &store,
        &FixedClock(jiff::civil::date(2026, 9, 18)),
        &mut FixedRandom(3),
        &oid(1),
        Side::Ours,
    )
    .unwrap();
    let unpushed: Vec<_> = store
        .unpushed(&remote())
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert!(
        unpushed.contains(&created.id),
        "the commit that already carried ours is still owed to the remote"
    );
}

#[test]
fn ours_keeps_the_local_version() {
    let store = conflicted();
    resolve(
        &store,
        &store,
        &store,
        &FixedClock(jiff::civil::date(2026, 9, 18)),
        &mut FixedRandom(3),
        &oid(1),
        Side::Ours,
    )
    .unwrap();
    assert_eq!(store.get(&oid(1)).unwrap().unwrap().base().subject, "mine");
    assert!(store.conflicts().unwrap().is_empty());
}

#[test]
fn resolving_a_non_conflict_is_refused() {
    let store = MemoryStore::new();
    let err = resolve(
        &store,
        &store,
        &store,
        &FixedClock(jiff::civil::date(2026, 9, 18)),
        &mut FixedRandom(3),
        &oid(1),
        Side::Ours,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(_))
    ));
}

fn config() -> Config {
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

fn upstream(subject: &str) -> IncomingObject {
    IncomingObject {
        remote_id: "r1".into(),
        object: Object::Task(Task::new(oid(0), subject)),
    }
}

/// Answers a pull with `objects` and a push by taking every mutation.
fn launcher(objects: Vec<IncomingObject>) -> ScriptedLauncher {
    recording_launcher(objects).0
}

fn recording_launcher(
    objects: Vec<IncomingObject>,
) -> (ScriptedLauncher, Rc<RefCell<Vec<RemoteMutation>>>) {
    let pushed = Rc::new(RefCell::new(Vec::new()));
    let seen = pushed.clone();
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: PullOutcome {
                objects: objects.clone(),
                ..PullOutcome::default()
            },
            push_answer: Box::new(|ms| {
                ms.iter()
                    .map(|m| MutationOutcome {
                        oid: m.oid.clone(),
                        ok: true,
                        remote_id: Some("r1".into()),
                        why: None,
                    })
                    .collect()
            }),
            pushed: seen.clone(),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    (l, pushed)
}

/// A real conflict: an object both sides moved away from the same base.
fn pulled_into_conflict() -> MemoryStore {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(9),
        "base",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    let mut mine = store.get(&oid(1)).unwrap().unwrap();
    mine.base_mut().subject = "mine".into();
    store.put(&mine).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(10),
        "mine",
    )
    .unwrap();
    let reports = pull(
        repos,
        &launcher(vec![upstream("theirs")]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].conflicts, 1);
    store
}

/// Finding core 1-2. Clearing the flag settled nothing: no commit carried
/// ours, and the tracking snapshot still held theirs, so every later pull
/// raised the identical conflict again.
#[test]
fn ours_is_committed_and_the_conflict_is_not_raised_again() {
    let store = pulled_into_conflict();
    let repos = Repositories::of(&store);
    resolve(
        &store,
        &store,
        &store,
        &FixedClock(date(2026, 9, 19)),
        &mut FixedRandom(3),
        &oid(1),
        Side::Ours,
    )
    .unwrap();

    let owed = store.unpushed(&remote()).unwrap();
    let resolution = owed
        .iter()
        .find(|c| c.message.contains("with ours"))
        .expect("the resolution is a commit of its own");
    assert_eq!(
        resolution.changes[0].after.as_ref().unwrap().base().subject,
        "mine"
    );

    // Pulling the same upstream object again, before the resolution has
    // reached the remote, is what used to re-raise it.
    let again = pull(
        repos,
        &launcher(vec![upstream("theirs")]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 20)),
        &mut FixedRandom(43),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(again[0].conflicts, 0, "the conflict was raised again");
    assert_eq!(store.get(&oid(1)).unwrap().unwrap().base().subject, "mine");

    let reports = push(repos, &launcher(vec![]), &EchoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].sent, 1, "ours was sent upstream");
    assert_eq!(
        store
            .remote_snapshot(&remote(), &oid(1))
            .unwrap()
            .unwrap()
            .base()
            .subject,
        "mine",
        "the tracking snapshot moved off theirs"
    );
}

/// The symmetric check. Theirs wins locally, and what is still owed upstream
/// has to end at theirs too, or the next push sends ours back and undoes the
/// resolution.
#[test]
fn theirs_leaves_only_theirs_owed_and_settles_the_conflict() {
    let store = pulled_into_conflict();
    let repos = Repositories::of(&store);
    resolve(
        &store,
        &store,
        &store,
        &FixedClock(date(2026, 9, 19)),
        &mut FixedRandom(3),
        &oid(1),
        Side::Theirs,
    )
    .unwrap();
    let (l, sent) = recording_launcher(vec![]);
    push(repos, &l, &EchoCredentials, &config(), None).unwrap();
    for m in sent.borrow().iter() {
        assert_eq!(
            m.object.as_ref().unwrap().base().subject,
            "theirs",
            "a push after resolving with theirs must not carry ours"
        );
    }
    let again = pull(
        repos,
        &launcher(vec![upstream("theirs")]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 20)),
        &mut FixedRandom(43),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(again[0].conflicts, 0);
    assert_eq!(
        store.get(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
}
