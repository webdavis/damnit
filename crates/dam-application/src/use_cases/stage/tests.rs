use super::*;
use crate::errors::Refusal;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Fixture, MemoryStore, oid};
use dam_domain::{Object, Op, Task};

fn seed(store: &MemoryStore, byte: u8) -> Oid {
    store
        .put(&Object::Task(Task::new(oid(byte), format!("t{byte}"))))
        .unwrap();
    oid(byte)
}

#[test]
fn add_stages_a_create_for_a_new_working_object() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let staged = dam.add(&[id]).unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].op(), Op::Create);
    assert_eq!(store.staged().unwrap().len(), 1);
}

#[test]
fn add_skips_an_unchanged_object() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let record = dam_domain::CommitRecord {
        id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
        message: "m".into(),
        at: jiff::Timestamp::UNIX_EPOCH,
        changes: dam.add(std::slice::from_ref(&id)).unwrap(),
    };
    store.commit(&record).unwrap();
    assert!(dam.add(&[id]).unwrap().is_empty());
}

#[test]
fn add_unstages_an_object_edited_back_to_its_committed_state() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let record = dam_domain::CommitRecord {
        id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
        message: "m".into(),
        at: jiff::Timestamp::UNIX_EPOCH,
        changes: dam.add(std::slice::from_ref(&id)).unwrap(),
    };
    store.commit(&record).unwrap();
    store
        .put(&Object::Task(Task::new(id.clone(), "edited")))
        .unwrap();
    dam.add(std::slice::from_ref(&id)).unwrap();
    store
        .put(&Object::Task(Task::new(id.clone(), "t1")))
        .unwrap();
    assert!(dam.add(std::slice::from_ref(&id)).unwrap().is_empty());
    assert!(store.staged().unwrap().is_empty());
}

#[test]
fn add_all_includes_deletes_of_committed_objects_gone_from_working() {
    let store = MemoryStore::new();
    let id = seed(&store, 1);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let record = dam_domain::CommitRecord {
        id: dam_domain::CommitId::generate(&mut |b| b.fill(9)),
        message: "m".into(),
        at: jiff::Timestamp::UNIX_EPOCH,
        changes: dam.add(std::slice::from_ref(&id)).unwrap(),
    };
    store.commit(&record).unwrap();
    store.delete(&id).unwrap();
    let staged = dam.add_all().unwrap();
    assert_eq!(staged[0].op(), Op::Delete);
}

#[test]
fn reset_unstages_one_or_all() {
    let store = MemoryStore::new();
    let a = seed(&store, 1);
    let b = seed(&store, 2);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    dam.add_all().unwrap();
    dam.reset(&[a]).unwrap();
    assert_eq!(store.staged().unwrap().len(), 1);
    dam.reset(&[]).unwrap();
    assert!(store.staged().unwrap().is_empty());
    let _ = b;
}

#[test]
fn add_of_an_unknown_oid_is_refused() {
    let store = MemoryStore::new();
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    let err = dam.add(&[oid(7)]).unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(_))
    ));
}

#[test]
fn add_that_fails_between_two_objects_stages_neither() {
    let store = FaultyStore::new(MemoryStore::new());
    let a = seed(&store.inner, 1);
    let b = seed(&store.inner, 2);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.add(&[a.clone(), b.clone()]).map(drop));
}

#[test]
fn add_all_that_fails_between_two_objects_stages_neither() {
    let store = FaultyStore::new(MemoryStore::new());
    seed(&store.inner, 1);
    seed(&store.inner, 2);
    let fixture = Fixture::default();
    let dam = fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.add_all().map(drop));
}
