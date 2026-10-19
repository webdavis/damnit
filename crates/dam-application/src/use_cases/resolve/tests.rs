mod round_trip;

use super::*;
use crate::ports::Conflict;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Scenario, oid, todoist};
use dam_domain::{Object, Task};

fn conflict_over_mine() -> Conflict {
    Conflict {
        remote: todoist(),
        oid: oid(1),
        ours: Object::Task(Task::new(oid(1), "mine")),
        theirs: Object::Task(Task::new(oid(1), "theirs")),
    }
}

fn conflicted() -> Scenario {
    let s = Scenario::default();
    s.working(Object::Task(Task::new(oid(1), "mine")));
    s.store.mark_conflict(&conflict_over_mine()).unwrap();
    s
}

#[test]
fn theirs_takes_the_upstream_version() {
    let s = conflicted();
    s.dam().resolve(&oid(1), Side::Theirs).unwrap();
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
    assert_eq!(
        s.store.committed(&oid(1)).unwrap().unwrap().base().subject,
        "theirs"
    );
    assert!(s.store.conflicts().unwrap().is_empty());
    let owed = s.store.unpushed(&todoist()).unwrap();
    assert_eq!(
        owed[0].changes[0].after().unwrap().base().subject,
        "theirs",
        "what is owed upstream ends at theirs, so a push cannot send ours back"
    );
}

#[test]
fn resolving_with_ours_leaves_an_existing_unpushed_commit_untouched() {
    let s = Scenario::default();
    s.committed(Object::Task(Task::new(oid(1), "mine")));
    let created = s.store.log().unwrap()[0].id.clone();
    s.store.mark_conflict(&conflict_over_mine()).unwrap();
    s.dam().resolve(&oid(1), Side::Ours).unwrap();
    let unpushed: Vec<_> = s
        .store
        .unpushed(&todoist())
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert!(
        unpushed.contains(&created),
        "the commit that already carried ours is still owed to the remote"
    );
}

#[test]
fn ours_keeps_the_local_version() {
    let s = conflicted();
    s.dam().resolve(&oid(1), Side::Ours).unwrap();
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "mine"
    );
    assert!(s.store.conflicts().unwrap().is_empty());
}

#[test]
fn resolving_a_non_conflict_is_refused() {
    let s = Scenario::default();
    let err = s.dam().resolve(&oid(1), Side::Ours).unwrap_err();
    assert_eq!(err, UseCaseError::Refused(Refusal::NoConflict(oid(1))));
}

#[test]
fn resolve_that_fails_between_its_writes_changes_nothing() {
    let s = conflicted();
    let store = FaultyStore::new(s.store.clone());
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.resolve(&oid(1), Side::Theirs));
}
