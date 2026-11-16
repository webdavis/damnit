use dam_domain::Object;
use dam_domain::Permission::{RemotePush, TaskRead};

use super::except::excepting_kitchen;
use super::parents::held_by;
use crate::service::Dam;
use crate::testing::{
    Scenario, accepting_all, allowed, holder, oid, pushing, remotes_launched_by, task_in,
};
use crate::use_cases::push::PushReport;
use crate::use_cases::sight::HELD_BY_WHAT_YOU_CANNOT_SEE;

fn a_child_held_by_a_parent_in_the_kitchen() -> Scenario {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "P", "kitchen/")))
        .committed(Object::Task(held_by(2, "c", "Home/", 1)));
    s
}

fn pushed_by(s: &Scenario, paths: Option<&[&str]>) -> PushReport {
    let token = holder(vec![
        allowed(&[RemotePush], None),
        allowed(&[TaskRead], paths),
    ]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    let (l, _) = pushing(accepting_all(None));
    dam.push(remotes_launched_by(&l), None).unwrap().remove(0)
}

#[test]
fn a_hold_whose_reason_names_a_hidden_parent_is_told_in_the_fixed_reason() {
    let report = pushed_by(&a_child_held_by_a_parent_in_the_kitchen(), Some(&["Home"]));
    assert_eq!(
        report.failed,
        [(oid(2), HELD_BY_WHAT_YOU_CANNOT_SEE.to_string())]
    );
}

#[test]
fn a_hold_whose_reason_names_only_what_the_token_reads_is_shown_as_it_is() {
    let report = pushed_by(&a_child_held_by_a_parent_in_the_kitchen(), None);
    let (_, why) = &report.failed[0];
    assert!(why.contains("remote.todoist.except"), "{why}");
}
