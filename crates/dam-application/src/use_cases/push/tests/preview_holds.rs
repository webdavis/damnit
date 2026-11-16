use dam_domain::Object;
use dam_domain::Permission::{RemotePush, TaskRead};

use super::except::excepting_kitchen;
use super::parents::held_by;
use crate::access::Holder;
use crate::ports::Declared;
use crate::service::Dam;
use crate::testing::prelude::*;
use crate::testing::{Scenario, allowed, holder, oid, task_caps, task_in, todoist};

const GENERIC: &str = "held until a task you cannot see is settled";

fn on_the_wire(_: &crate::remote::RemoteMutation) -> Result<Declared, crate::ports::HelperError> {
    Ok(Declared::new())
}

fn child_of_an_excepted_parent() -> Scenario {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(7, "P", "kitchen/")))
        .committed(Object::Task(held_by(6, "c", "Home/", 7)));
    s.store
        .set_accepted_kinds(&todoist(), &task_caps().kinds)
        .unwrap();
    s.store
        .set_accepted_fields(&todoist(), &task_caps().fields)
        .unwrap();
    s
}

fn reading(paths: &[&str]) -> Holder {
    holder(vec![
        allowed(&[RemotePush], None),
        allowed(&[TaskRead], Some(paths)),
    ])
}

fn held_reason(s: &Scenario, token: &Holder) -> String {
    let dam = Dam {
        access: token,
        ..s.dam()
    };
    let previews = dam.push_preview(None, &on_the_wire).unwrap();
    let plan = previews[0].plan.as_ref().unwrap();
    assert_eq!(plan.held.len(), 1);
    assert_eq!(plan.held[0].oid, oid(6));
    plan.held[0].why.clone()
}

#[test]
fn a_hold_on_a_hidden_parent_in_the_working_copy_does_not_name_it() {
    let s = child_of_an_excepted_parent();
    assert_eq!(held_reason(&s, &reading(&["Home"])), GENERIC);
}

#[test]
fn a_hold_on_a_hidden_parent_left_only_in_the_last_commit_does_not_name_it() {
    let s = child_of_an_excepted_parent();
    s.store.delete(&oid(7)).unwrap();
    assert_eq!(held_reason(&s, &reading(&["Home"])), GENERIC);
}

#[test]
fn a_hold_on_a_visible_parent_keeps_its_full_reason() {
    let s = child_of_an_excepted_parent();
    let why = held_reason(&s, &reading(&["Home", "kitchen"]));
    assert!(why.contains("remote.todoist.except"), "{why}");
}
