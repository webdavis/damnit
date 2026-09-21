use super::*;
use crate::testing::prelude::*;
use crate::testing::{FixedClock, FixedRandom, MemoryStore, oid};
use crate::use_cases::commit::commit;
use crate::use_cases::stage::add;
use dam_domain::{Object, Task};
use jiff::civil::date;

/// A task put, staged and committed, so it has a commit behind it.
fn committed_task(store: &MemoryStore, byte: u8) -> Oid {
    let id = oid(byte);
    store
        .put(&Object::Task(Task::new(id.clone(), format!("t{byte}"))))
        .unwrap();
    add(store, store, std::slice::from_ref(&id)).unwrap();
    commit(
        store,
        store,
        &FixedClock(date(2026, 9, 18)),
        &FixedRandom::new(byte),
        "m",
    )
    .unwrap();
    id
}

fn retitle(store: &MemoryStore, oid: &Oid, subject: &str) {
    let mut object = store.get(oid).unwrap().unwrap();
    object.base_mut().subject = subject.into();
    store.put(&object).unwrap();
}

#[test]
fn a_working_edit_goes_back_to_the_last_commit() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    let out = restore(&store, &store, std::slice::from_ref(&id)).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].changed);
    assert_eq!(store.get(&id).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn a_working_delete_comes_back() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    store.delete(&id).unwrap();
    restore(&store, &store, std::slice::from_ref(&id)).unwrap();
    assert_eq!(store.get(&id).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn a_staged_change_is_unstaged_and_discarded_together() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    add(&store, &store, std::slice::from_ref(&id)).unwrap();
    assert_eq!(store.staged().unwrap().len(), 1);
    restore(&store, &store, std::slice::from_ref(&id)).unwrap();
    assert!(store.staged().unwrap().is_empty());
    assert_eq!(store.get(&id).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn an_object_with_no_commit_behind_it_is_refused_naming_it() {
    let store = MemoryStore::new();
    let id = oid(7);
    store
        .put(&Object::Task(Task::new(id.clone(), "fresh")))
        .unwrap();
    let err = restore(&store, &store, std::slice::from_ref(&id)).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NotCommitted(id.clone()))
    );
    assert!(err.to_string().contains(id.short()), "{err}");
}

#[test]
fn an_unchanged_object_is_a_no_op_that_says_so() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    let out = restore(&store, &store, std::slice::from_ref(&id)).unwrap();
    assert!(!out[0].changed);
    assert_eq!(store.get(&id).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn a_refusal_leaves_every_other_named_object_alone() {
    let store = MemoryStore::new();
    let good = committed_task(&store, 1);
    let fresh = oid(7);
    store
        .put(&Object::Task(Task::new(fresh.clone(), "fresh")))
        .unwrap();
    retitle(&store, &good, "edited");
    restore(&store, &store, &[good.clone(), fresh]).unwrap_err();
    assert_eq!(
        store.get(&good).unwrap().unwrap().base().subject,
        "edited",
        "the whole command refuses before any object is written"
    );
}

/// Restore means the working copy equals the commit and nothing about it is
/// staged, so a stage left over from an edit that was typed back by hand is
/// cleared even though the working copy already matches.
#[test]
fn a_stale_stage_is_cleared_even_when_the_working_copy_already_matches() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    add(&store, &store, std::slice::from_ref(&id)).unwrap();
    retitle(&store, &id, "t1");
    let out = restore(&store, &store, std::slice::from_ref(&id)).unwrap();
    assert!(out[0].changed);
    assert!(store.staged().unwrap().is_empty());
}

/// Two mutually exclusive lists naming the same object is a document no client
/// can read, and the second pass would always find the work already done.
#[test]
fn a_repeated_oid_is_restored_once() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    let out = restore(&store, &store, &[id.clone(), id.clone()]).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].changed);
}
