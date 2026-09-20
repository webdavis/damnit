use jiff::civil::date;

use super::tests::{config, launcher, remote};
use super::*;
use crate::ports::Repositories;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, NoCredentials, oid};
use crate::wire::to_wire;
use dam_domain::{Event, Kind, Object, Task, When};
use dam_protocol::PullResponse;

#[test]
fn a_kind_change_upstream_keeps_ours_and_raises_a_notice() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    let mut theirs = to_wire(
        &Object::Event(Event::new(
            oid(1),
            "theirs",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )),
        Some("r1".into()),
    );
    theirs.oid = String::new();
    let l = launcher(PullResponse {
        objects: vec![theirs],
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
    let mut theirs = to_wire(
        &Object::Event(Event::new(
            oid(1),
            "theirs",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        )),
        Some("r1".into()),
    );
    theirs.oid = String::new();
    let l = launcher(PullResponse {
        objects: vec![theirs],
        removed: vec![],
        sync: None,
    });
    for _ in 0..2 {
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
    }
    assert_eq!(store.notices().unwrap().len(), 1);
}
