use dam_domain::Outcome;
use dam_domain::Permission::{TaskCreate, TaskRead, TaskRemove};

use super::{added, as_holder, dentist, dentist_task, moved_home, sent, tool, waiting_removal};
use crate::errors::{Refusal, UseCaseError};
use crate::testing::Scenario;
use crate::testing::prelude::*;
use crate::use_cases::remove::{ItemAndTask, Removal};

#[test]
fn an_item_still_waiting_under_the_id_is_withdrawn() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate]);
    let (hash, oid) = sent(&s, &tool);
    assert_eq!(
        as_holder(&s, &tool).remove_by_id(&dentist()).unwrap(),
        Removal::Withdrawn(ItemAndTask {
            item: hash.clone(),
            oid
        })
    );
    let item = s.store.item(&hash).unwrap().unwrap();
    assert!(matches!(item.outcome, Outcome::Withdrawn(_)));
}

#[test]
fn an_added_task_gets_one_removal_item_however_often_the_id_is_sent() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let oid = added(&s, &tool);
    let dam = as_holder(&s, &tool);
    let first = waiting_removal(dam.remove_by_id(&dentist()).unwrap());
    assert_eq!(first.oid, oid);
    assert_eq!(
        waiting_removal(dam.remove_by_id(&dentist()).unwrap()),
        first
    );
    assert_eq!(s.dam().waiting_items().unwrap().len(), 1);
    s.dam().dismiss(&first.item).unwrap();
    assert_eq!(
        dam.remove_by_id(&dentist()).unwrap(),
        Removal::Declined(first)
    );
    assert!(s.store.get(&oid).unwrap().is_some());
}

#[test]
fn an_id_that_leads_to_nothing_of_the_callers_answers_nothing() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let dam = as_holder(&s, &tool);
    assert_eq!(dam.remove_by_id(&dentist()).unwrap(), Removal::Nothing);
    sent(&s, &tool);
    dam.remove_by_id(&dentist()).unwrap();
    assert_eq!(dam.remove_by_id(&dentist()).unwrap(), Removal::Nothing);
}

#[test]
fn a_task_the_owner_removed_himself_answers_nothing() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let oid = added(&s, &tool);
    s.dam().remove(&oid).unwrap();
    let dam = as_holder(&s, &tool);
    assert_eq!(dam.remove_by_id(&dentist()).unwrap(), Removal::Nothing);
}

#[test]
fn a_task_outside_the_grants_answers_nothing() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let oid = added(&s, &tool);
    moved_home(&s, &oid);
    let dam = as_holder(&s, &tool);
    assert_eq!(dam.remove_by_id(&dentist()).unwrap(), Removal::Nothing);
    assert!(s.dam().waiting_items().unwrap().is_empty());
}

#[test]
fn sending_a_removal_by_id_needs_task_remove() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead]);
    added(&s, &tool);
    assert_eq!(
        as_holder(&s, &tool).remove_by_id(&dentist()),
        Err(UseCaseError::Refused(Refusal::NotPermitted(TaskRemove)))
    );
}

#[test]
fn the_owner_removes_his_own_task_by_id_and_hears_nothing_for_an_unknown_id() {
    let s = Scenario::default().drawing_from(7);
    let oid = s
        .dam()
        .new_task(dentist_task())
        .unwrap()
        .written()
        .oid()
        .clone();
    match s.dam().remove_by_id(&dentist()).unwrap() {
        Removal::Removed(plan) => assert_eq!(plan.oid, oid),
        other => panic!("not removed: {other:?}"),
    }
    assert!(s.store.get(&oid).unwrap().is_none());
    assert_eq!(s.dam().remove_by_id(&dentist()).unwrap(), Removal::Nothing);
}
