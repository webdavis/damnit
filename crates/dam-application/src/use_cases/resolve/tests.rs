use super::*;
use crate::ports::RemoteName;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add_all;
use dam_domain::{Object, Task};

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
    assert!(store.unpushed(&remote()).unwrap().is_empty());
}

#[test]
fn resolving_with_ours_leaves_an_existing_unpushed_commit_untouched() {
    let store = MemoryStore::new();
    store.put(&Object::Task(Task::new(oid(1), "mine"))).unwrap();
    add_all(&store).unwrap();
    let created = commit(
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
    assert_eq!(unpushed, vec![created.id]);
}

#[test]
fn ours_keeps_the_local_version() {
    let store = conflicted();
    resolve(
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
