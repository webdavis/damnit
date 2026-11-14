use dam_domain::Oid;
use dam_domain::Permission::{TaskCreate, TaskRead, TaskRemove};

use super::{added, as_holder, moved_home, tool, waiting_removal};
use crate::errors::{Refusal, UseCaseError};
use crate::testing::Scenario;
use crate::testing::prelude::*;
use crate::use_cases::remove::Removal;

#[test]
fn a_tokens_removal_of_an_added_task_waits_as_one_item_and_the_task_stays() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let oid = added(&s, &tool);
    let dam = as_holder(&s, &tool);
    let first = waiting_removal(dam.remove_or_send(&oid).unwrap());
    let again = waiting_removal(dam.remove_or_send(&oid).unwrap());
    assert_eq!(first.oid, oid);
    assert_eq!(again, first);
    let waiting = s.dam().waiting_items().unwrap();
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].hash, first.item);
    assert_eq!(waiting[0].kind().as_str(), "removal");
    assert!(s.store.get(&oid).unwrap().is_some());
}

#[test]
fn a_removal_the_owner_declined_answers_declined_and_is_not_sent_again() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let oid = added(&s, &tool);
    let dam = as_holder(&s, &tool);
    let removal = waiting_removal(dam.remove_or_send(&oid).unwrap());
    s.dam().dismiss(&removal.item).unwrap();
    assert_eq!(
        dam.remove_or_send(&oid).unwrap(),
        Removal::Declined(removal)
    );
    assert!(s.dam().waiting_items().unwrap().is_empty());
    assert!(s.store.get(&oid).unwrap().is_some());
}

#[test]
fn sending_a_removal_needs_task_remove() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead]);
    let oid = added(&s, &tool);
    assert_eq!(
        as_holder(&s, &tool).remove_or_send(&oid),
        Err(UseCaseError::Refused(Refusal::NotPermitted(TaskRemove)))
    );
    assert!(s.dam().waiting_items().unwrap().is_empty());
}

#[test]
fn a_task_outside_the_grants_answers_as_a_missing_one_does() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate, TaskRead, TaskRemove]);
    let oid = added(&s, &tool);
    moved_home(&s, &oid);
    let dam = as_holder(&s, &tool);
    let missing = Oid::generate(&mut |b| b.fill(99));
    for named in [&oid, &missing] {
        assert_eq!(
            dam.remove_or_send(named),
            Err(UseCaseError::Refused(Refusal::NoSuchObject(
                named.short().to_string()
            )))
        );
    }
}

#[test]
fn the_owners_removal_by_oid_still_removes_at_once() {
    let s = Scenario::default().drawing_from(7);
    let tool = tool(&[TaskCreate]);
    let oid = added(&s, &tool);
    assert!(matches!(
        s.dam().remove_or_send(&oid).unwrap(),
        Removal::Removed(_)
    ));
    assert!(s.store.get(&oid).unwrap().is_none());
}
