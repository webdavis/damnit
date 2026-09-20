use jiff::civil::date;

use super::tests::{config, launcher, remote};
use super::*;
use crate::ports::Repositories;
use crate::remote::{IncomingObject, PullOutcome};
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, MemoryStore, oid};
use dam_domain::{Event, Kind, Object, Task, When};

#[test]
fn a_kind_change_upstream_keeps_ours_and_raises_a_notice() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    let theirs = IncomingObject {
        remote_id: "r1".into(),
        object: Object::Event(Event::new(
            oid(1),
            "theirs",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )),
    };
    let l = launcher(PullOutcome {
        objects: vec![theirs],
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
    assert_eq!(reports[0].unchanged, 1);
    assert_eq!(
        store.get(&oid(1)).unwrap(),
        Some(Object::Task(Task::new(oid(1), "mine")))
    );
    assert_eq!(
        store.notices().unwrap(),
        vec![Notice::KindChanged {
            oid: oid(1),
            ours: Kind::Task,
            theirs: Kind::Event
        }]
    );
}

#[test]
fn a_kind_change_is_noticed_once_however_often_it_is_pulled() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    let theirs = IncomingObject {
        remote_id: "r1".into(),
        object: Object::Event(Event::new(
            oid(1),
            "theirs",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )),
    };
    let l = launcher(PullOutcome {
        objects: vec![theirs],
        removed: vec![],
        ..PullOutcome::default()
    });
    for _ in 0..2 {
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
    }
    assert_eq!(store.notices().unwrap().len(), 1);
}
