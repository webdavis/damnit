use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::*;
use crate::config::{CredentialSpec, RemoteConfig};
use crate::testing::{
    FixedClock, FixedRandom, MemoryStore, NoCredentials, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use crate::wire::to_wire;
use dam_domain::{Object, Task};
use dam_protocol::{PullResponse, WireObject};

fn remote() -> RemoteName {
    RemoteName("todoist".into())
}

fn config() -> Config {
    Config {
        remotes: vec![RemoteConfig {
            name: remote(),
            helper: "todoist".into(),
            credentials: vec![CredentialSpec::Literal {
                name: "api_token".into(),
                value: "t".into(),
            }],
            stale: None,
            path: None,
        }],
        ..Config::default()
    }
}

fn launcher(answer: PullResponse) -> ScriptedLauncher {
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

fn wire(subject: &str, remote_id: &str) -> WireObject {
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
    let l = launcher(PullResponse {
        objects: vec![wire("from todoist", "r1")],
        removed: vec![],
        sync: Some("s1".into()),
    });
    let reports = pull(
        &store,
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
    store.put(&Object::Task(Task::new(oid(1), "old"))).unwrap();
    add_all(&store).unwrap();
    commit(
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
        &store,
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
    let mut t = Task::new(oid(1), "old");
    t.base.depends.push(oid(7));
    store.put(&Object::Task(t.clone())).unwrap();
    add_all(&store).unwrap();
    commit(
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
        &store,
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
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    add_all(&store).unwrap();
    commit(
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
    add_all(&store).unwrap();
    commit(
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
        &store,
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
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    add_all(&store).unwrap();
    commit(
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
        &store,
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

#[test]
fn a_removal_upstream_is_a_notice_not_a_deletion() {
    let store = MemoryStore::new();
    store
        .put(&Object::Task(Task::new(oid(1), "keep me")))
        .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "keep me")))
        .unwrap();
    let l = launcher(PullResponse {
        objects: vec![],
        removed: vec!["r1".into()],
        sync: None,
    });
    let reports = pull(
        &store,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].removed_upstream, 1);
    assert!(store.get(&oid(1)).unwrap().is_some());
    assert!(matches!(
        store.notices().unwrap()[0],
        Notice::RemovedUpstream { .. }
    ));
    assert!(store.oid_for_remote_id(&remote(), "r1").unwrap().is_none());
    assert!(store.remote_snapshot(&remote(), &oid(1)).unwrap().is_none());
}

#[test]
fn a_cancelled_event_with_attached_tasks_is_reported() {
    let store = MemoryStore::new();
    let event = dam_domain::Event::new(
        oid(5),
        "meeting",
        dam_domain::When::Day(date(2026, 1, 1)),
        dam_domain::When::Day(date(2026, 1, 2)),
    );
    store.put(&Object::Event(event.clone())).unwrap();
    store.map_remote_id(&remote(), &oid(5), "e1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Event(event.clone()))
        .unwrap();
    let mut attached = Task::new(oid(6), "prep");
    attached.event = Some(oid(5));
    store.put(&Object::Task(attached)).unwrap();
    let mut cancelled = event.clone();
    cancelled.status = dam_domain::EventStatus::Cancelled;
    let mut w = to_wire(&Object::Event(cancelled), Some("e1".into()));
    w.oid = String::new();
    let mut caps = task_caps();
    caps.kinds = vec!["event".into()];
    caps.fields = vec![
        "subject".into(),
        "status".into(),
        "start".into(),
        "end".into(),
    ];
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: PullResponse {
                objects: vec![w.clone()],
                removed: vec![],
                sync: None,
            },
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    pull(
        &store,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap();
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
    assert!(store.get(&oid(6)).unwrap().is_some());
}

#[test]
fn an_unrelated_staged_object_stays_staged_after_a_pull() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    add_all(&store).unwrap();
    commit(
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
    store
        .put(&Object::Task(Task::new(oid(2), "unrelated")))
        .unwrap();
    add_all(&store).unwrap();
    let l = launcher(PullResponse {
        objects: vec![wire("new", "r1")],
        removed: vec![],
        sync: None,
    });
    pull(
        &store,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        store
            .staged()
            .unwrap()
            .iter()
            .map(|c| c.oid.clone())
            .collect::<Vec<_>>(),
        vec![oid(2)]
    );
}

#[test]
fn a_cancelled_event_that_conflicts_still_raises_the_notice() {
    let store = MemoryStore::new();
    let event = dam_domain::Event::new(
        oid(5),
        "meeting",
        dam_domain::When::Day(date(2026, 1, 1)),
        dam_domain::When::Day(date(2026, 1, 2)),
    );
    store.put(&Object::Event(event.clone())).unwrap();
    add_all(&store).unwrap();
    commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(20),
        "m",
    )
    .unwrap();
    store.map_remote_id(&remote(), &oid(5), "e1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Event(event.clone()))
        .unwrap();

    let mut edited = event.clone();
    edited.base.subject = "moved".into();
    store.put(&Object::Event(edited)).unwrap();
    add_all(&store).unwrap();
    commit(
        &store,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(21),
        "m2",
    )
    .unwrap();

    let mut attached = Task::new(oid(6), "prep");
    attached.event = Some(oid(5));
    store.put(&Object::Task(attached)).unwrap();

    let mut cancelled = event.clone();
    cancelled.status = dam_domain::EventStatus::Cancelled;
    let mut w = to_wire(&Object::Event(cancelled), Some("e1".into()));
    w.oid = String::new();
    let mut caps = task_caps();
    caps.kinds = vec!["event".into()];
    caps.fields = vec![
        "subject".into(),
        "status".into(),
        "start".into(),
        "end".into(),
    ];
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: PullResponse {
                objects: vec![w.clone()],
                removed: vec![],
                sync: None,
            },
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
    let reports = pull(
        &store,
        &l,
        &NoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &mut FixedRandom(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].conflicts, 1);
    assert_eq!(store.conflicts().unwrap().len(), 1);
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
}
