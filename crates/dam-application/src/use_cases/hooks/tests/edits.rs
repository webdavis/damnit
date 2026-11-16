use std::path::PathBuf;

use dam_domain::Oid;
use dam_domain::Permission::TaskCreate;

use super::{every_hook, hooked, new_task, ran, retitled};
use crate::config::HookEvent::{TaskChanged, TaskNew};
use crate::config::OnFailure;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{EditRequest, HookFailure};
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid};
use crate::use_cases::edit::EditFields;
use crate::use_cases::items::land::Landed;
use crate::use_cases::prefix::Named;

fn request() -> EditRequest {
    EditRequest {
        json: r#"{"labels": ["errand"]}"#.into(),
        file: PathBuf::from("/home/ada/.config/dam/config.lua"),
        line: Some(3),
    }
}

pub(super) fn labelled(label: &str) -> EditFields {
    EditFields {
        add_labels: vec![label.into()],
        ..EditFields::default()
    }
}

pub(super) fn hook_returns(s: &Scenario, read: Result<EditFields, String>) {
    *s.fixture.hooks.requesting.borrow_mut() = Some(request());
    *s.fixture.edit_requests.reading.borrow_mut() = Some(read);
}

fn labels(s: &Scenario, oid: &Oid) -> Vec<String> {
    let object = s.store.get(oid).unwrap().unwrap();
    object.base().labels.iter().cloned().collect()
}

fn warned_why(s: &Scenario, why: &str) {
    assert_eq!(
        *s.fixture.hook_warnings.warned.borrow(),
        [HookFailure {
            event: TaskNew,
            failure: request().failed(why.into()),
        }]
    );
}

#[test]
fn the_edit_a_hook_returns_lands_on_its_task_in_the_same_command() {
    let s = hooked(&[(TaskNew, None)]);
    hook_returns(&s, Ok(labelled("errand")));
    let landed = s.dam().new_task(new_task("call the dentist")).unwrap();
    assert_eq!(labels(&s, landed.oid()), ["errand"]);
    assert_eq!(
        *s.fixture.hook_edits.edited.borrow(),
        [(TaskNew, landed.oid().clone())]
    );
    assert_eq!(
        *s.fixture.edit_requests.read.borrow(),
        [(landed.oid().clone(), request())]
    );
}

#[test]
fn the_edit_a_hook_returns_runs_no_further_hook() {
    let s = every_hook();
    hook_returns(&s, Ok(labelled("errand")));
    s.dam().new_task(new_task("call the dentist")).unwrap();
    assert_eq!(ran(&s), [(TaskNew, "call the dentist")]);
}

#[test]
fn a_wrong_request_that_warns_keeps_the_commands_change_and_names_what_was_wrong() {
    let s = hooked(&[(TaskNew, Some(OnFailure::Warn))]);
    let why = "--request: colour is not a field it takes";
    hook_returns(&s, Err(why.into()));
    let landed = s.dam().new_task(new_task("call the dentist")).unwrap();
    assert!(labels(&s, landed.oid()).is_empty());
    warned_why(&s, why);
    assert!(s.fixture.hook_edits.edited.borrow().is_empty());
}

#[test]
fn a_wrong_request_that_refuses_changes_nothing() {
    let s = hooked(&[(TaskChanged, Some(OnFailure::Refuse))]);
    let why = "--request: colour is not a field it takes";
    hook_returns(&s, Err(why.into()));
    assert_eq!(
        s.dam().edit(&oid(1), retitled("pay the rent")),
        Err(UseCaseError::Refused(Refusal::HookRefused(HookFailure {
            event: TaskChanged,
            failure: request().failed(why.into()),
        })))
    );
    assert_eq!(
        s.store.get(&oid(1)).unwrap().unwrap().base().subject,
        "pay rent"
    );
}

#[test]
fn an_edit_the_edit_rules_refuse_is_a_failure_with_that_refusal() {
    let s = hooked(&[(TaskNew, Some(OnFailure::Warn))]);
    hook_returns(
        &s,
        Ok(EditFields {
            undone: true,
            ..labelled("errand")
        }),
    );
    let landed = s.dam().new_task(new_task("call the dentist")).unwrap();
    assert!(labels(&s, landed.oid()).is_empty());
    warned_why(
        &s,
        &format!("{:?}", Refusal::NotCompleted(landed.oid().clone())),
    );
}

#[test]
fn an_edit_that_changes_nothing_is_no_failure_and_no_change() {
    let s = hooked(&[(TaskNew, Some(OnFailure::Refuse))]);
    hook_returns(&s, Ok(retitled("call the dentist")));
    let landed = s.dam().new_task(new_task("call the dentist")).unwrap();
    assert!(s.store.get(landed.oid()).unwrap().is_some());
    assert!(s.fixture.hook_warnings.warned.borrow().is_empty());
    assert!(s.fixture.hook_edits.edited.borrow().is_empty());
}

#[test]
fn adding_a_tokens_item_applies_the_edit_its_hook_returns() {
    let s = every_hook();
    let tool = holder(vec![allowed(&[TaskCreate], None)]);
    let as_tool = Dam {
        access: &tool,
        ..s.dam()
    };
    let Landed::Sent(sent) = as_tool.new_task(new_task("call the dentist")).unwrap() else {
        panic!("a token's new task was written, not sent");
    };
    hook_returns(&s, Ok(labelled("errand")));
    s.dam().add(&[Named::Item(sent.item.hash.clone())]).unwrap();
    assert_eq!(labels(&s, sent.item.hash.oid()), ["errand"]);
    assert_eq!(ran(&s), [(TaskNew, "call the dentist")]);
}
