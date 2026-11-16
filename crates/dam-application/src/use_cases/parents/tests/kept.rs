use dam_domain::{Object, Path};

use super::pulled::pull;
use super::{errands, parent_of};
use crate::ports::CommitRepository;
use crate::testing::{Scenario, oid, task_in};
use crate::use_cases::edit::EditFields;

fn milk_under_the_second_of_two_errands() -> Scenario {
    let s = errands();
    s.filed(task_in(9, "errands", "Home"));
    let mut milk = task_in(2, "milk", "Home/errands");
    milk.base.parent = Some(oid(9));
    s.committed(Object::Task(milk)).tracked_as(&oid(2), "r1");
    s
}

#[test]
fn an_edit_of_the_subject_or_body_keeps_a_parent_its_namesakes_would_hide() {
    let s = milk_under_the_second_of_two_errands();
    let edit = EditFields {
        subject: Some("oat milk".into()),
        body: Some("two cartons".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(2), edit).unwrap();
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(9)));
}

#[test]
fn a_pull_that_leaves_the_path_alone_keeps_a_parent_its_namesakes_would_hide() {
    let s = milk_under_the_second_of_two_errands();
    let mut theirs = task_in(0, "milk", "Home/errands");
    theirs.base.body = "oat".into();
    pull(&s, Object::Task(theirs));
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(9)));
}

#[test]
fn a_move_finds_the_parent_again() {
    let s = milk_under_the_second_of_two_errands();
    s.dam()
        .relocate(&oid(2), &Path::parse("Home").unwrap())
        .unwrap();
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(5)));
}

#[test]
fn a_kept_parent_that_is_gone_is_found_again() {
    let s = errands();
    let mut milk = task_in(2, "milk", "Home/errands");
    milk.base.parent = Some(oid(7));
    s.filed(milk);
    let edit = EditFields {
        body: Some("two cartons".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(2), edit).unwrap();
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
}

#[test]
fn a_reverted_removal_whose_parent_is_gone_finds_its_parent_again() {
    let s = errands();
    let mut milk = task_in(2, "milk", "Home/errands");
    milk.base.parent = Some(oid(7));
    s.committed(Object::Task(milk)).deleted(&oid(2));
    let removal = s.store.log().unwrap()[0].id.to_string();
    s.dam().revert(&removal, &[]).unwrap();
    assert_eq!(parent_of(&s, &oid(2)), Some(oid(1)));
}
