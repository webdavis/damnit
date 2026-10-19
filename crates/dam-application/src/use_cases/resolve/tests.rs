mod round_trip;

use super::*;
use crate::ports::{Conflict, RemoteName};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, FixedRandom, Fixture, MemoryStore, oid};
use dam_domain::{Object, Task};

pub(super) fn remote() -> RemoteName {
    RemoteName("todoist".into())
}

fn conflicted() -> MemoryStore {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    store
        .mark_conflict(&Conflict {
            remote: remote(),
            oid: oid(1),
            ours: Object::Task(Task::new(oid(1), "mine")),
            theirs: Object::Task(Task::new(oid(1), "theirs")),
        })
        .unwrap();
    store
}

#[test]
fn theirs_takes_the_upstream_version() {
    let store = conflicted();
    let fixture = Fixture {
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.resolve(&oid(1), Side::Theirs).unwrap();
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
        owed[0].changes[0].after().unwrap().base().subject,
        "theirs",
        "what is owed upstream ends at theirs, so a push cannot send ours back"
    );
}

#[test]
fn resolving_with_ours_leaves_an_existing_unpushed_commit_untouched() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    let created = Dam {
        random: &FixedRandom::new(1),
        ..dam
    }
    .commit("m")
    .unwrap();
    store
        .mark_conflict(&Conflict {
            remote: remote(),
            oid: oid(1),
            ours: Object::Task(Task::new(oid(1), "mine")),
            theirs: Object::Task(Task::new(oid(1), "theirs")),
        })
        .unwrap();
    Dam {
        random: &FixedRandom::new(3),
        ..dam
    }
    .resolve(&oid(1), Side::Ours)
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
    let fixture = Fixture {
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    dam.resolve(&oid(1), Side::Ours).unwrap();
    assert_eq!(store.get(&oid(1)).unwrap().unwrap().base().subject, "mine");
    assert!(store.conflicts().unwrap().is_empty());
}

#[test]
fn resolving_a_non_conflict_is_refused() {
    let store = MemoryStore::new();
    let fixture = Fixture {
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    let err = dam.resolve(&oid(1), Side::Ours).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NoConflict(oid(1))));
    assert_eq!(err.to_string(), "0101010 has no conflict to resolve");
}

#[test]
fn resolve_that_fails_between_its_writes_changes_nothing() {
    let store = FaultyStore::new(conflicted());
    let fixture = Fixture {
        random: FixedRandom::new(3),
        ..Fixture::default()
    };
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.resolve(&oid(1), Side::Theirs));
}
