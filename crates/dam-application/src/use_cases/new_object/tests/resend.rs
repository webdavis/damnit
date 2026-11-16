use dam_domain::CallerId;
use dam_domain::Permission::{TaskChange, TaskCreate, TaskRead};

use super::parent::{milk, two_errands};
use super::*;
use crate::ports::ObjectRepository;
use crate::testing::{Scenario, allowed, holder};

fn by_id(id: &str, path: &str, parent: Option<u8>) -> NewTask {
    NewTask {
        id: Some(CallerId::parse(id).unwrap()),
        ..milk(path, parent)
    }
}

fn as_token(
    s: &Scenario,
    permissions: &[dam_domain::Permission],
    task: NewTask,
) -> Result<Landed, UseCaseError> {
    let tool = holder(vec![allowed(permissions, None)]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    dam.new_task(task)
}

fn sent_parent(landed: Landed) -> Option<Oid> {
    let Landed::Sent(sent) = landed else {
        panic!("a token's task was written");
    };
    sent.item.change.object().base().parent.clone()
}

#[test]
fn a_token_holding_change_but_not_create_resends_its_own_task() {
    let s = two_errands();
    as_token(
        &s,
        &[TaskCreate, TaskChange, TaskRead],
        by_id("milk", "Work", None),
    )
    .unwrap();
    let mut again = by_id("milk", "Work", None);
    again.priority = dam_domain::Priority::HIGHEST;
    as_token(&s, &[TaskChange, TaskRead], again).unwrap();
}

#[test]
fn a_token_holding_change_but_not_create_is_refused_a_new_id() {
    let s = two_errands();
    let err = as_token(&s, &[TaskChange, TaskRead], by_id("milk", "Work", None)).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NotPermitted(TaskCreate))
    );
}

#[test]
fn a_tokens_resend_naming_no_parent_keeps_the_parent_its_task_has() {
    let s = two_errands();
    let all = [TaskCreate, TaskChange, TaskRead];
    as_token(&s, &all, by_id("milk", "", Some(2))).unwrap();
    let mut again = by_id("milk", "Home/errands", None);
    again.body = "oat".into();
    let landed = as_token(&s, &all, again).unwrap();
    assert_eq!(sent_parent(landed), Some(oid(2)));
}

#[test]
fn the_owners_resend_naming_no_parent_keeps_the_parent_and_the_place() {
    let s = two_errands();
    let landed = s.dam().new_task(by_id("milk", "", Some(2))).unwrap();
    let milk = written_oid(landed);
    let mut again = by_id("milk", "Home/errands", None);
    again.body = "oat".into();
    s.dam().new_task(again).unwrap();
    let object = s.store.get(&milk).unwrap().unwrap();
    assert_eq!(
        (object.base().path.as_str(), object.base().parent.clone()),
        ("Home/errands/", Some(oid(2)))
    );
}
