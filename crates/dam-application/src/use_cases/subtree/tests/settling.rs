use dam_domain::Object;

use super::*;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::edit::EditFields;

fn parent_of(s: &Scenario, oid: &Oid) -> Option<Oid> {
    s.store.get(oid).unwrap().unwrap().base().parent.clone()
}

fn one_errands_holding_an_unsettled_task() -> Scenario {
    let s = Scenario::default().drawing_from(42);
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(1, "errands", "Home"));
    s.store
        .put(&Object::Task(task_in(3, "stamps", "Home/errands")))
        .unwrap();
    s
}

#[test]
fn removing_the_only_possible_parent_moves_its_unsettled_content_up() {
    let s = one_errands_holding_an_unsettled_task();
    let grandparent = parent_of(&s, &oid(1));
    s.dam().remove(&oid(1)).unwrap();
    assert_eq!(s.path_of(&oid(3)), "Home/");
    assert_eq!(parent_of(&s, &oid(3)), grandparent);
}

#[test]
fn moving_the_only_possible_parent_carries_and_settles_its_unsettled_content() {
    let s = one_errands_holding_an_unsettled_task();
    s.dam()
        .relocate(&oid(1), &Path::parse("Work").unwrap())
        .unwrap();
    assert_eq!(s.path_of(&oid(3)), "Work/errands/");
    assert_eq!(parent_of(&s, &oid(3)), Some(oid(1)));
}

#[test]
fn renaming_the_only_possible_parent_carries_and_settles_its_unsettled_content() {
    let s = one_errands_holding_an_unsettled_task();
    let rename = EditFields {
        subject: Some("chores".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(1), rename).unwrap();
    assert_eq!(s.path_of(&oid(3)), "Home/chores/");
    assert_eq!(parent_of(&s, &oid(3)), Some(oid(1)));
}

#[test]
fn completing_the_only_possible_parent_is_blocked_by_its_unsettled_open_task() {
    let s = one_errands_holding_an_unsettled_task();
    let blockers = s.dam().plan_complete(&oid(1)).unwrap().blockers;
    assert_eq!(blockers.children, vec![oid(3)]);
}

#[test]
fn a_rename_that_names_no_folder_is_refused_while_an_unsettled_task_sits_inside() {
    let s = one_errands_holding_an_unsettled_task();
    let rename = EditFields {
        subject: Some("..".into()),
        ..EditFields::default()
    };
    let refused = s.dam().edit(&oid(1), rename).unwrap_err();
    assert!(
        matches!(
            refused,
            UseCaseError::Refused(Refusal::SubjectNamesNoFolder(_))
        ),
        "{refused:?}"
    );
    assert_eq!(s.path_of(&oid(3)), "Home/errands/");
}
