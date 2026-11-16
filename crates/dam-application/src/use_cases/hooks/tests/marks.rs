use dam_domain::Oid;

use super::edits::{hook_returns, labelled};
use super::{hooked, new_task, retitled};
use crate::config::HookEvent::{TaskChanged, TaskNew};
use crate::ports::HookMarkRepository;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid};
use crate::use_cases::prefix::Named;

fn hook_named_in_the_stage(s: &Scenario) -> Option<String> {
    let staged = s.store.staged().unwrap();
    staged[0].details.as_ref()?.meta.get("hook").cloned()
}

fn edited_by_its_hook() -> Scenario {
    let s = hooked(&[(TaskChanged, None)]);
    hook_returns(&s, Ok(labelled("errand")));
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    s
}

fn add(s: &Scenario, oid: &Oid) {
    s.dam().add(&[Named::Object(oid.clone())]).unwrap();
}

#[test]
fn a_hook_that_changes_a_task_marks_it_with_its_event() {
    let s = edited_by_its_hook();
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), Some(TaskChanged));
}

#[test]
fn a_hook_that_returns_no_edit_leaves_no_mark() {
    let s = hooked(&[(TaskChanged, None)]);
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), None);
}

#[test]
fn the_latest_hook_to_change_a_task_is_the_one_its_mark_names() {
    let s = hooked(&[(TaskNew, None), (TaskChanged, None)]);
    hook_returns(&s, Ok(labelled("errand")));
    let landed = s.dam().new_task(new_task("call the dentist")).unwrap();
    hook_returns(&s, Ok(labelled("phone")));
    s.dam()
        .edit(landed.oid(), retitled("call the dentist today"))
        .unwrap();
    assert_eq!(s.store.hooked(landed.oid()).unwrap(), Some(TaskChanged));
}

#[test]
fn adding_a_task_a_hook_changed_names_the_hook_in_the_staged_change() {
    let s = edited_by_its_hook();
    add(&s, &oid(1));
    assert_eq!(hook_named_in_the_stage(&s).as_deref(), Some("task_changed"));
}

#[test]
fn adding_a_task_no_hook_changed_names_no_hook() {
    let s = hooked(&[(TaskChanged, None)]);
    s.dam().edit(&oid(1), retitled("pay the rent")).unwrap();
    add(&s, &oid(1));
    assert_eq!(hook_named_in_the_stage(&s), None);
}

#[test]
fn a_commit_records_the_hook_and_forgets_the_mark() {
    let s = edited_by_its_hook();
    add(&s, &oid(1));
    let record = s.dam().commit("rent").unwrap();
    let details = record.changes[0].details.as_ref().unwrap();
    assert_eq!(
        details.meta.get("hook").map(String::as_str),
        Some("task_changed")
    );
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), None);
}

#[test]
fn a_reset_keeps_the_mark() {
    let s = edited_by_its_hook();
    add(&s, &oid(1));
    s.dam().reset(&[oid(1)]).unwrap();
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), Some(TaskChanged));
}

#[test]
fn a_restore_forgets_the_mark() {
    let s = edited_by_its_hook();
    s.dam().restore(&[oid(1)]).unwrap();
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), None);
}

#[test]
fn a_removal_forgets_the_mark() {
    let s = edited_by_its_hook();
    s.dam().remove(&oid(1)).unwrap();
    assert_eq!(s.store.hooked(&oid(1)).unwrap(), None);
}
