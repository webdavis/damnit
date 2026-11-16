use dam_domain::Object;
use dam_domain::Permission::{TaskChange, TaskComplete, TaskRead};

use super::*;
use crate::service::Dam;
use crate::testing::{Scenario, allowed, holder, oid, task_in};
use crate::use_cases::edit::EditFields;

fn shared(byte: u8, subject: &str, path: &str, seen: &[u8]) -> Object {
    let mut task = task_in(byte, subject, path);
    if seen.contains(&byte) {
        task.base.labels.insert("shared".into());
    }
    Object::Task(task)
}

fn home_over_two_errands_and_an_unsettled_task(seen: &[u8]) -> Scenario {
    let s = Scenario::default();
    s.working(shared(5, "Home", "", seen))
        .working(shared(1, "errands", "Home", seen))
        .working(shared(2, "errands", "Home", seen));
    s.store
        .put(&shared(3, "stamps", "Home/errands", seen))
        .unwrap();
    s
}

fn tool() -> crate::access::Holder {
    let mut grant = allowed(&[TaskRead, TaskComplete, TaskChange], None);
    grant.labels = Some(["shared".to_string()].into());
    holder(vec![grant])
}

fn tokens_done(seen: &[u8]) -> UseCaseError {
    let s = home_over_two_errands_and_an_unsettled_task(seen);
    let tool = tool();
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    dam.plan_complete(&oid(5)).err().unwrap()
}

fn hidden_home() -> UseCaseError {
    UseCaseError::Refused(Refusal::NoSuchObject(oid(5).short().to_string()))
}

#[test]
fn a_token_that_sees_none_of_what_blocks_done_is_answered_as_if_the_object_were_missing() {
    assert_eq!(tokens_done(&[5]), hidden_home());
}

#[test]
fn a_token_that_sees_part_of_what_blocks_done_is_answered_as_if_the_object_were_missing() {
    assert_eq!(tokens_done(&[5, 1, 3]), hidden_home());
}

#[test]
fn a_token_that_sees_all_of_what_blocks_done_is_told_which_namesakes_hold_it() {
    assert!(matches!(
        tokens_done(&[5, 1, 2, 3]),
        UseCaseError::Refused(Refusal::ParentNotKnown { namesakes, unsettled, .. })
            if namesakes == vec![oid(1), oid(2)] && unsettled == vec![oid(3)]
    ));
}

#[test]
fn a_token_renaming_an_object_over_a_hidden_namesake_is_answered_as_if_it_were_missing() {
    let s = home_over_two_errands_and_an_unsettled_task(&[5]);
    let tool = tool();
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let rename = EditFields {
        subject: Some("..".into()),
        ..EditFields::default()
    };
    assert_eq!(dam.edit(&oid(5), rename).unwrap_err(), hidden_home());
}
