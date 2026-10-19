use super::*;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, FixedRandom, Fixture, MemoryStore, oid};
use dam_domain::{Object, Task};

fn committed_task(store: &MemoryStore, byte: u8) -> Oid {
    let id = oid(byte);
    store
        .put(&Object::Task(Task::new(id.clone(), format!("t{byte}"))))
        .unwrap();
    let fixture = Fixture {
        random: FixedRandom::new(byte),
        ..Fixture::default()
    };
    let dam = fixture.dam(store);
    dam.add(std::slice::from_ref(&id)).unwrap();
    dam.commit("m").unwrap();
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
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let out = dam.restore(std::slice::from_ref(&id)).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].changed);
    assert_eq!(store.get(&id).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn a_working_delete_comes_back() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    store.delete(&id).unwrap();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.restore(std::slice::from_ref(&id)).unwrap();
    assert_eq!(store.get(&id).unwrap().unwrap().base().subject, "t1");
}

#[test]
fn a_staged_change_is_unstaged_and_discarded_together() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add(std::slice::from_ref(&id)).unwrap();
    assert_eq!(store.staged().unwrap().len(), 1);
    dam.restore(std::slice::from_ref(&id)).unwrap();
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
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let err = dam.restore(std::slice::from_ref(&id)).unwrap_err();
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
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let out = dam.restore(std::slice::from_ref(&id)).unwrap();
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
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.restore(&[good.clone(), fresh]).unwrap_err();
    assert_eq!(
        store.get(&good).unwrap().unwrap().base().subject,
        "edited",
        "the whole command refuses before any object is written"
    );
}

#[test]
fn a_stale_stage_is_cleared_even_when_the_working_copy_already_matches() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add(std::slice::from_ref(&id)).unwrap();
    retitle(&store, &id, "t1");
    let out = dam.restore(std::slice::from_ref(&id)).unwrap();
    assert!(out[0].changed);
    assert!(store.staged().unwrap().is_empty());
}

#[test]
fn a_repeated_oid_is_restored_once() {
    let store = MemoryStore::new();
    let id = committed_task(&store, 1);
    retitle(&store, &id, "edited");
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let out = dam.restore(&[id.clone(), id.clone()]).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].changed);
}

#[test]
fn an_oid_neither_layer_knows_is_no_such_object() {
    let store = MemoryStore::new();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let err = dam.restore(&[oid(9)]).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(oid(9).short().to_string()))
    );
}

#[test]
fn restore_that_fails_between_two_objects_restores_neither() {
    let store = FaultyStore::new(MemoryStore::new());
    let a = committed_task(&store.inner, 1);
    let b = committed_task(&store.inner, 2);
    retitle(&store.inner, &a, "edited a");
    retitle(&store.inner, &b, "edited b");
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.restore(&[a.clone(), b.clone()]).map(drop));
}
