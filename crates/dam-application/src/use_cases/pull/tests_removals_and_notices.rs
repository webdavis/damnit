use std::cell::RefCell;
use std::rc::Rc;

use jiff::civil::date;

use super::tests::{config, incoming, launcher, remote};
use super::*;
use crate::ports::Repositories;
use crate::remote::{IncomingObject, PullOutcome, RejectedObject};
use crate::testing::prelude::*;
use crate::testing::{
    EchoCredentials, FixedClock, FixedRandom, MemoryStore, ScriptedHelper, ScriptedLauncher, oid,
    task_caps,
};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Field, Kind, Object, Task};

#[test]
fn a_removal_upstream_is_a_notice_not_a_deletion() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store
        .put(&Object::Task(Task::new(oid(1), "keep me")))
        .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "keep me")))
        .unwrap();
    let l = launcher(PullOutcome {
        objects: vec![],
        removed: vec!["r1".into()],
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
    let repos = Repositories::of(&store);
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
    let w = IncomingObject {
        remote_id: "e1".into(),
        object: Object::Event(cancelled),
    };
    let mut caps = task_caps();
    caps.kinds = vec![Kind::Event];
    caps.fields = vec![Field::Subject, Field::Status, Field::Start, Field::End];
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: PullOutcome {
                objects: vec![w.clone()],
                ..PullOutcome::default()
            },
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
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
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
    assert!(store.get(&oid(6)).unwrap().is_some());
}

#[test]
fn an_unrelated_staged_object_stays_staged_after_a_pull() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
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
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    store
        .put(&Object::Task(Task::new(oid(2), "unrelated")))
        .unwrap();
    add_all(&store, &store, &store).unwrap();
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
    let repos = Repositories::of(&store);
    let event = dam_domain::Event::new(
        oid(5),
        "meeting",
        dam_domain::When::Day(date(2026, 1, 1)),
        dam_domain::When::Day(date(2026, 1, 2)),
    );
    store.put(&Object::Event(event.clone())).unwrap();
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(20),
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
    add_all(&store, &store, &store).unwrap();
    commit(
        &store,
        &store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(21),
        "m2",
    )
    .unwrap();

    let mut attached = Task::new(oid(6), "prep");
    attached.event = Some(oid(5));
    store.put(&Object::Task(attached)).unwrap();

    let mut cancelled = event.clone();
    cancelled.status = dam_domain::EventStatus::Cancelled;
    let w = IncomingObject {
        remote_id: "e1".into(),
        object: Object::Event(cancelled),
    };
    let mut caps = task_caps();
    caps.kinds = vec![Kind::Event];
    caps.fields = vec![Field::Subject, Field::Status, Field::Start, Field::End];
    let l = ScriptedLauncher {
        make: Box::new(move || ScriptedHelper {
            caps: caps.clone(),
            pull_answer: PullOutcome {
                objects: vec![w.clone()],
                ..PullOutcome::default()
            },
            push_answer: Box::new(|_| vec![]),
            pushed: Rc::new(RefCell::new(vec![])),
            pulled_since: Rc::new(RefCell::new(vec![])),
        }),
        launched_with: Rc::new(RefCell::new(vec![])),
    };
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
    assert_eq!(reports[0].conflicts, 1);
    assert_eq!(store.conflicts().unwrap().len(), 1);
    assert!(matches!(
        store.notices().unwrap().last(),
        Some(Notice::EventCancelled { attached: 1, .. })
    ));
}

#[test]
fn one_unconvertible_object_is_skipped_with_a_notice_and_the_rest_still_pull() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let l = launcher(PullOutcome {
        objects: vec![incoming("from todoist", "r-good")],
        rejected: vec![RejectedObject {
            remote_id: "r-bad".into(),
            why: "path: cannot read \"a//b/\"".into(),
        }],
        removed: vec![],
        sync: Some("s1".into()),
    });
    let reports = pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(1),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].created, 1);
    let notices = store.notices().unwrap();
    assert!(
        notices
            .iter()
            .any(|n| matches!(n, Notice::PullFailed { why, .. } if why.contains("r-bad")))
    );
    assert!(
        store
            .oid_for_remote_id(&remote(), "r-bad")
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_pull_records_when_it_happened() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    let l = launcher(PullOutcome::default());
    let clock = FixedClock(date(2026, 9, 18));
    pull(
        repos,
        &l,
        &EchoCredentials,
        &clock,
        &FixedRandom::new(1),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(store.last_pull(&remote()).unwrap(), Some(clock.now()));
}
