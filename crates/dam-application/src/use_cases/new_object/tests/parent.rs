use super::*;
use crate::errors::Retry;
use crate::ports::ObjectRepository;
use crate::testing::{Scenario, task_in};

pub(super) fn two_errands() -> Scenario {
    let s = Scenario::default().drawing_from(9);
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(1, "errands", "Home"))
        .filed(task_in(2, "errands", "Home"));
    s
}

pub(super) fn milk(path: &str, parent: Option<u8>) -> NewTask {
    NewTask {
        parent: parent.map(oid),
        path: Path::parse(path).unwrap(),
        ..input("buy milk", &[])
    }
}

fn placed_at(s: &Scenario, landed: Landed) -> (String, Option<Oid>) {
    let object = s.store.get(&written_oid(landed)).unwrap().unwrap();
    (
        object.base().path.as_str().to_string(),
        object.base().parent.clone(),
    )
}

#[test]
fn a_new_task_given_a_parent_lands_in_its_folder_with_that_parent() {
    let s = two_errands();
    let landed = s.dam().new_task(milk("", Some(2))).unwrap();
    assert_eq!(
        placed_at(&s, landed),
        ("Home/errands/".into(), Some(oid(2)))
    );
}

#[test]
fn a_new_task_given_a_parent_and_a_path_inside_its_folder_keeps_both() {
    let s = two_errands();
    let landed = s
        .dam()
        .new_task(milk("Home/errands/later", Some(2)))
        .unwrap();
    assert_eq!(
        placed_at(&s, landed),
        ("Home/errands/later/".into(), Some(oid(2)))
    );
}

#[test]
fn a_new_task_given_a_parent_and_a_path_outside_its_folder_is_refused() {
    let s = two_errands();
    let err = s.dam().new_task(milk("Work", Some(2))).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::PathOutsideParent {
            parent: oid(2),
            folder: Path::parse("Home/errands").unwrap(),
            path: Path::parse("Work").unwrap(),
        })
    );
}

#[test]
fn a_new_task_at_a_path_two_namesakes_hold_is_refused_until_one_is_named() {
    let s = two_errands();
    let err = s.dam().new_task(milk("Home/errands", None)).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::SeveralHoldThePath {
            namesakes: vec![oid(1), oid(2)],
            folder: Path::parse("Home/errands").unwrap(),
            retry: Retry::New("buy milk".into()),
        })
    );
    assert!(s.store.get(&oid(9)).unwrap().is_none());
}

#[test]
fn a_new_task_with_an_id_at_a_path_two_namesakes_hold_is_refused() {
    let s = two_errands();
    let task = NewTask {
        id: Some(dam_domain::CallerId::parse("milk-1").unwrap()),
        ..milk("Home/errands", None)
    };
    let err = s.dam().new_task(task).unwrap_err();
    assert!(matches!(
        err,
        UseCaseError::Refused(Refusal::SeveralHoldThePath { .. })
    ));
}
