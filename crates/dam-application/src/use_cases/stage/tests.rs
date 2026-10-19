use super::*;
use crate::errors::Refusal;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Scenario, oid, task};
use dam_domain::{Object, Op, Task};

#[test]
fn add_stages_a_create_for_a_new_working_object() {
    let s = Scenario::default();
    s.working(task(1));
    let staged = s.dam().add(&[oid(1)]).unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0].op(), Op::Create);
    assert_eq!(s.store.staged().unwrap().len(), 1);
}

#[test]
fn add_skips_an_unchanged_object() {
    let s = Scenario::default();
    s.committed(task(1));
    assert!(s.dam().add(&[oid(1)]).unwrap().is_empty());
}

#[test]
fn add_unstages_an_object_edited_back_to_its_committed_state() {
    let s = Scenario::default();
    s.committed(task(1));
    s.working(Object::Task(Task::new(oid(1), "edited")));
    let dam = s.dam();
    dam.add(&[oid(1)]).unwrap();
    s.working(task(1));
    assert!(dam.add(&[oid(1)]).unwrap().is_empty());
    assert!(s.store.staged().unwrap().is_empty());
}

#[test]
fn add_all_includes_deletes_of_committed_objects_gone_from_working() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store.delete(&oid(1)).unwrap();
    let staged = s.dam().add_all().unwrap();
    assert_eq!(staged[0].op(), Op::Delete);
}

#[test]
fn add_all_after_a_committed_removal_stages_nothing() {
    let s = Scenario::default();
    s.committed(task(1)).deleted(&oid(1));
    assert!(s.dam().add_all().unwrap().is_empty());
    assert!(s.store.staged().unwrap().is_empty());
}

#[test]
fn reset_unstages_one_or_all() {
    let s = Scenario::default();
    s.working(task(1)).working(task(2));
    let dam = s.dam();
    dam.add_all().unwrap();
    dam.reset(&[oid(1)]).unwrap();
    assert_eq!(s.store.staged().unwrap().len(), 1);
    dam.reset(&[]).unwrap();
    assert!(s.store.staged().unwrap().is_empty());
}

#[test]
fn add_of_an_unknown_oid_is_refused() {
    let s = Scenario::default();
    let err = s.dam().add(&[oid(7)]).unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(_))
    ));
}

#[test]
fn add_that_fails_between_two_objects_stages_neither() {
    let s = Scenario::default();
    s.working(task(1)).working(task(2));
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.add(&[oid(1), oid(2)]).map(drop));
}

#[test]
fn add_all_that_fails_between_two_objects_stages_neither() {
    let s = Scenario::default();
    s.working(task(1)).working(task(2));
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.add_all().map(drop));
}
