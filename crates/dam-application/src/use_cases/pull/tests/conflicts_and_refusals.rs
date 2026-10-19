use jiff::civil::date;

use super::super::*;
use super::{config, incoming, launcher, remote};
use crate::errors::Refusal;
use crate::ports::Repositories;
use crate::remote::{PullOutcome, RejectedObject};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, Fixture, MemoryStore, oid};
use dam_domain::{Object, Task};

#[test]
fn both_sides_changed_is_a_conflict_and_nothing_is_overwritten() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    Dam {
        random: &FixedRandom::new(9),
        ..dam
    }
    .commit("m")
    .unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    let mut mine = store.get(&oid(1)).unwrap().unwrap();
    mine.base_mut().subject = "mine".into();
    store.put(&mine).unwrap();
    dam.add_all().unwrap();
    Dam {
        random: &FixedRandom::new(10),
        ..dam
    }
    .commit("local moves and is committed")
    .unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("theirs", "r1")],
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
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.commit("m").unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    let mut dirty = store.get(&oid(1)).unwrap().unwrap();
    dirty.base_mut().subject = "unsaved".into();
    store.put(&dirty).unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("theirs", "r1"), incoming("brand new", "r2")],
        rejected: vec![],
        removed: vec![],
        cancelled: vec![],
        sync: Some("s9".into()),
    });
    let err = pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
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
fn a_refused_pull_leaves_none_of_its_own_notices_behind() {
    let store = MemoryStore::new();
    let repos = Repositories::of(&store);
    store.put(&Object::Task(Task::new(oid(1), "base"))).unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(9),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.commit("m").unwrap();
    store.map_remote_id(&remote(), &oid(1), "r1").unwrap();
    store
        .set_remote_snapshot(&remote(), &Object::Task(Task::new(oid(1), "base")))
        .unwrap();
    let mut dirty = store.get(&oid(1)).unwrap().unwrap();
    dirty.base_mut().subject = "unsaved".into();
    store.put(&dirty).unwrap();
    let l = launcher(PullOutcome {
        objects: vec![incoming("theirs", "r1")],
        rejected: vec![RejectedObject {
            remote_id: "r-bad".into(),
            why: "path: cannot read \"a//b\"".into(),
        }],
        removed: vec![],
        cancelled: vec![],
        sync: Some("s9".into()),
    });
    let err = pull(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::DirtyOnPull { oid: oid(1) })
    );
    assert!(
        store.notices().unwrap().is_empty(),
        "the rejection notice was written before the refusal and rolled back with it"
    );
}
