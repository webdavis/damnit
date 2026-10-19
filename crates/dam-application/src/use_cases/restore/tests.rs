use super::*;
use crate::testing::prelude::*;
use crate::testing::{FaultyStore, Scenario, oid, task};
use dam_domain::{Object, Task};

fn retitled(s: &Scenario, byte: u8, subject: &str) {
    s.working(Object::Task(Task::new(oid(byte), subject)));
}

fn subject_of(s: &Scenario, byte: u8) -> String {
    s.store
        .get(&oid(byte))
        .unwrap()
        .unwrap()
        .base()
        .subject
        .clone()
}

#[test]
fn a_working_edit_goes_back_to_the_last_commit() {
    let s = Scenario::default();
    s.committed(task(1));
    retitled(&s, 1, "edited");
    let out = s.dam().restore(&[oid(1)]).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].changed);
    assert_eq!(subject_of(&s, 1), "t1");
}

#[test]
fn a_working_delete_comes_back() {
    let s = Scenario::default();
    s.committed(task(1));
    s.store.delete(&oid(1)).unwrap();
    s.dam().restore(&[oid(1)]).unwrap();
    assert_eq!(subject_of(&s, 1), "t1");
}

#[test]
fn a_staged_change_is_unstaged_and_discarded_together() {
    let s = Scenario::default();
    s.committed(task(1));
    retitled(&s, 1, "edited");
    let dam = s.dam();
    dam.add(&[oid(1)]).unwrap();
    assert_eq!(s.store.staged().unwrap().len(), 1);
    dam.restore(&[oid(1)]).unwrap();
    assert!(s.store.staged().unwrap().is_empty());
    assert_eq!(subject_of(&s, 1), "t1");
}

#[test]
fn an_object_with_no_commit_behind_it_is_refused_naming_it() {
    let s = Scenario::default();
    retitled(&s, 7, "fresh");
    let id = oid(7);
    let err = s.dam().restore(std::slice::from_ref(&id)).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NotCommitted(id.clone()))
    );
}

#[test]
fn an_unchanged_object_is_a_no_op_that_says_so() {
    let s = Scenario::default();
    s.committed(task(1));
    let out = s.dam().restore(&[oid(1)]).unwrap();
    assert!(!out[0].changed);
    assert_eq!(subject_of(&s, 1), "t1");
}

#[test]
fn a_refusal_leaves_every_other_named_object_alone() {
    let s = Scenario::default();
    s.committed(task(1));
    retitled(&s, 7, "fresh");
    retitled(&s, 1, "edited");
    s.dam().restore(&[oid(1), oid(7)]).unwrap_err();
    assert_eq!(
        subject_of(&s, 1),
        "edited",
        "the whole command refuses before any object is written"
    );
}

#[test]
fn a_stale_stage_is_cleared_even_when_the_working_copy_already_matches() {
    let s = Scenario::default();
    s.committed(task(1));
    retitled(&s, 1, "edited");
    let dam = s.dam();
    dam.add(&[oid(1)]).unwrap();
    retitled(&s, 1, "t1");
    let out = dam.restore(&[oid(1)]).unwrap();
    assert!(out[0].changed);
    assert!(s.store.staged().unwrap().is_empty());
}

#[test]
fn a_repeated_oid_is_restored_once() {
    let s = Scenario::default();
    s.committed(task(1));
    retitled(&s, 1, "edited");
    let out = s.dam().restore(&[oid(1), oid(1)]).unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].changed);
}

#[test]
fn an_oid_neither_layer_knows_is_no_such_object() {
    let s = Scenario::default();
    let err = s.dam().restore(&[oid(9)]).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(oid(9).short().to_string()))
    );
}

#[test]
fn restore_that_fails_between_two_objects_restores_neither() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    retitled(&s, 1, "edited a");
    retitled(&s, 2, "edited b");
    let store = FaultyStore::new(s.store);
    let dam = s.fixture.dam(&store);
    store.assert_unchanged_when_any_write_fails(|| dam.restore(&[oid(1), oid(2)]).map(drop));
}
