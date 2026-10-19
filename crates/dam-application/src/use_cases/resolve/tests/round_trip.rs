mod doubles;

use super::super::*;
use super::remote;
use crate::ports::Repositories;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{EchoCredentials, FixedClock, FixedRandom, Fixture, MemoryStore, oid};
use crate::use_cases::pull::pull;
use crate::use_cases::push::push;
use dam_domain::{Object, Task};
use doubles::{
    config, launcher_that_accepts_every_push, recording_launcher_that_accepts_every_push, upstream,
};
use jiff::civil::date;

fn both_sides_moved_off_one_base() -> MemoryStore {
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
    .commit("base")
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
    .commit("mine")
    .unwrap();
    let reports = pull(
        repos,
        &launcher_that_accepts_every_push(vec![upstream("theirs")]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(42),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(reports[0].conflicts, 1);
    store
}

#[test]
fn ours_is_committed_and_the_conflict_is_not_raised_again() {
    let store = both_sides_moved_off_one_base();
    let repos = Repositories::of(&store);
    let fixture = Fixture {
        clock: FixedClock(date(2026, 9, 19)),
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.resolve(&oid(1), Side::Ours).unwrap();

    let owed = store.unpushed(&remote()).unwrap();
    let resolution = owed
        .iter()
        .find(|c| c.message.contains("with ours"))
        .expect("the resolution is a commit of its own");
    assert_eq!(
        resolution.changes[0].after().unwrap().base().subject,
        "mine"
    );

    let pulled_again_before_the_resolution_reached_the_remote = pull(
        repos,
        &launcher_that_accepts_every_push(vec![upstream("theirs")]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 20)),
        &FixedRandom::new(43),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        pulled_again_before_the_resolution_reached_the_remote[0].conflicts, 0,
        "the conflict was raised again"
    );
    assert_eq!(store.get(&oid(1)).unwrap().unwrap().base().subject, "mine");

    let reports = push(
        repos,
        &launcher_that_accepts_every_push(vec![]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &config(),
        None,
    )
    .unwrap();
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

#[test]
fn theirs_leaves_only_theirs_owed_and_settles_the_conflict() {
    let store = both_sides_moved_off_one_base();
    let repos = Repositories::of(&store);
    let fixture = Fixture {
        clock: FixedClock(date(2026, 9, 19)),
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.resolve(&oid(1), Side::Theirs).unwrap();
    let (l, sent) = recording_launcher_that_accepts_every_push(vec![]);
    push(
        repos,
        &l,
        &EchoCredentials,
        &FixedClock(date(2026, 9, 18)),
        &config(),
        None,
    )
    .unwrap();
    for m in sent.borrow().iter() {
        assert_eq!(
            m.object.as_ref().unwrap().base().subject,
            "theirs",
            "a push after resolving with theirs must not carry ours"
        );
    }
    let pulled_again_before_the_resolution_reached_the_remote = pull(
        repos,
        &launcher_that_accepts_every_push(vec![upstream("theirs")]),
        &EchoCredentials,
        &FixedClock(date(2026, 9, 20)),
        &FixedRandom::new(43),
        &config(),
        None,
    )
    .unwrap();
    assert_eq!(
        pulled_again_before_the_resolution_reached_the_remote[0].conflicts,
        0
    );
    assert_eq!(
        store.get(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
}
