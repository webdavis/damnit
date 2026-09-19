use std::cell::RefCell;
use std::rc::Rc;

use dam_domain::{Event, Object, Task, When};
use dam_protocol::{Mutation, MutationResult, PullResponse};
use jiff::civil::date;

use super::*;
use crate::config::{Config, CredentialSpec, RemoteConfig};
use crate::ports::{Notice, RemoteName};
use crate::testing::{
    FixedClock, FixedRandom, MemoryStore, NoCredentials, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;

fn config() -> Config {
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

fn launcher(
    answer: impl Fn(&[Mutation]) -> Vec<MutationResult> + Clone + 'static,
) -> (ScriptedLauncher, Rc<RefCell<Vec<Mutation>>>) {
    let pushed = Rc::new(RefCell::new(Vec::new()));
    let seen = pushed.clone();
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: task_caps(),
            pull_answer: PullResponse {
                objects: vec![],
                removed: vec![],
                sync: None,
            },
            push_answer: Box::new(answer.clone()),
            pushed: seen.clone(),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    (l, pushed)
}

fn committed_task(store: &MemoryStore, byte: u8) {
    store
        .put(&Object::Task(Task::new(oid(byte), format!("t{byte}"))))
        .unwrap();
    add_all(store).unwrap();
    commit(
        store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(byte),
        "m",
    )
    .unwrap();
}

#[test]
fn a_committed_create_is_sent_and_its_remote_id_is_mapped() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id: Some("r1".into()),
                why: None,
            })
            .collect()
    });
    let reports = push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    assert_eq!(pushed.borrow()[0].op, "create");
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
    assert!(
        l.launched_with
            .borrow()
            .iter()
            .any(|c| c == &vec![("api_token".to_string(), "value-of-api_token".to_string())])
    );
}

#[test]
fn a_failed_mutation_becomes_a_notice_and_a_retry() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationResult {
                oid: m.oid.clone(),
                ok: false,
                remote_id: None,
                why: Some("rate limited".into()),
            })
            .collect()
    });
    let reports = push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert_eq!(
        reports[0].failed,
        vec![(oid(1), "rate limited".to_string())]
    );
    assert_eq!(
        store.push_retries(&RemoteName("todoist".into())).unwrap(),
        vec![oid(1)]
    );
    assert!(matches!(
        store.notices().unwrap()[0],
        Notice::PushFailed { .. }
    ));
    // the commit is marked pushed; the retry set carries the failure forward
    assert!(
        store
            .unpushed(&RemoteName("todoist".into()))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_mutation_the_helper_never_answered_is_a_failure() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    committed_task(&store, 2);
    let (l, _) = launcher(|ms| {
        ms.iter()
            .filter(|m| m.oid == oid(1).to_string())
            .map(|m| MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id: Some("r1".into()),
                why: None,
            })
            .collect()
    });
    let reports = push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 1);
    assert_eq!(
        reports[0].failed,
        vec![(oid(2), "no answer from helper".to_string())]
    );
    assert_eq!(
        store.push_retries(&RemoteName("todoist".into())).unwrap(),
        vec![oid(2)]
    );
    assert!(matches!(
        store.notices().unwrap()[0],
        Notice::PushFailed { .. }
    ));
    let still_unpushed = store.unpushed(&RemoteName("todoist".into())).unwrap();
    assert_eq!(still_unpushed.len(), 1);
    assert!(still_unpushed[0].changes.iter().any(|c| c.oid == oid(2)));
}

#[test]
fn a_retry_is_sent_as_an_update_of_the_committed_state() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    store
        .map_remote_id(&RemoteName("todoist".into()), &oid(1), "r1")
        .unwrap();
    store
        .set_push_retries(&RemoteName("todoist".into()), &[oid(1)])
        .unwrap();
    let (l, pushed) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id: None,
                why: None,
            })
            .collect()
    });
    push(&store, &l, &NoCredentials, &config(), None).unwrap();
    let sent = pushed.borrow();
    assert!(
        sent.iter()
            .any(|m| m.op == "update" && m.remote_id.as_deref() == Some("r1"))
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
    let remote = RemoteName("todoist".into());
    store.put(&Object::Task(Task::new(oid(1), "t1"))).unwrap();
    add_all(&store).unwrap();
    let created = commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(1),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote, &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote, &Object::Task(Task::new(oid(1), "t1")))
        .unwrap();
    store.mark_pushed(&remote, &created.id).unwrap();

    store.delete(&oid(1)).unwrap();
    add_all(&store).unwrap();
    commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(2),
        "rm",
    )
    .unwrap();

    let (l, _) = launcher(|ms| {
        ms.iter()
            .map(|m| MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id: None,
                why: None,
            })
            .collect()
    });
    push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert!(store.remote_id(&remote, &oid(1)).unwrap().is_none());
    assert!(store.remote_snapshot(&remote, &oid(1)).unwrap().is_none());
}

#[test]
fn duplicate_and_unsent_results_are_ignored() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    let (l, _) = launcher(|_| {
        vec![
            MutationResult {
                oid: oid(1).to_string(),
                ok: false,
                remote_id: None,
                why: Some("first".into()),
            },
            MutationResult {
                oid: oid(1).to_string(),
                ok: false,
                remote_id: None,
                why: Some("second".into()),
            },
            MutationResult {
                oid: oid(9).to_string(),
                ok: false,
                remote_id: None,
                why: Some("unsent".into()),
            },
        ]
    });
    let reports = push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].succeeded, 0);
    assert_eq!(reports[0].failed.len(), 1);
    let retries = store.push_retries(&RemoteName("todoist".into())).unwrap();
    assert_eq!(retries, vec![oid(1)]);
    assert!(!retries.contains(&oid(9)));
}

#[test]
fn a_delete_with_no_remote_mapping_is_skipped() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "t1"))).unwrap();
    add_all(&store).unwrap();
    let created = commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(1),
        "m",
    )
    .unwrap();
    store
        .mark_pushed(&RemoteName("todoist".into()), &created.id)
        .unwrap();

    store.delete(&oid(1)).unwrap();
    add_all(&store).unwrap();
    commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(2),
        "rm",
    )
    .unwrap();

    let (l, pushed) = launcher(|_| vec![]);
    let reports = push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert_eq!(reports[0].sent, 0);
    assert!(pushed.borrow().is_empty());
}

#[test]
fn an_event_is_skipped_by_a_task_only_helper() {
    let store = MemoryStore::new();
    let e = Event::new(
        oid(2),
        "e",
        When::Day(date(2026, 1, 1)),
        When::Day(date(2026, 1, 2)),
    );
    store.put(&Object::Event(e)).unwrap();
    add_all(&store).unwrap();
    commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(3),
        "m",
    )
    .unwrap();
    let (l, pushed) = launcher(|_| vec![]);
    let reports = push(&store, &l, &NoCredentials, &config(), None).unwrap();
    assert_eq!(reports[0].skipped, 1);
    assert!(pushed.borrow().is_empty());
}

#[test]
fn unresolved_conflicts_block_push() {
    let store = MemoryStore::new();
    committed_task(&store, 1);
    store
        .mark_conflict(
            &RemoteName("todoist".into()),
            &oid(1),
            &Object::Task(Task::new(oid(1), "theirs")),
        )
        .unwrap();
    let (l, _) = launcher(|_| vec![]);
    let err = push(&store, &l, &NoCredentials, &config(), None).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::UnresolvedConflicts(1)));
}

#[test]
fn an_unknown_remote_is_refused() {
    let store = MemoryStore::new();
    let (l, _) = launcher(|_| vec![]);
    let err = push(&store, &l, &NoCredentials, &config(), Some("nope")).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchRemote("nope".into()))
    );
}
