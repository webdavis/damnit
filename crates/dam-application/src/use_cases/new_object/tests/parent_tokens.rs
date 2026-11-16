use dam_domain::Grant;
use dam_domain::Permission::{TaskCreate, TaskRead};

use super::parent::{milk, two_errands};
use super::*;
use crate::errors::Retry;
use crate::testing::{Scenario, allowed, event, holder};

fn token_new(s: &Scenario, grants: Vec<Grant>, task: NewTask) -> Result<Landed, UseCaseError> {
    let tool = holder(grants);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    dam.new_task(task)
}

fn not_permitted() -> UseCaseError {
    UseCaseError::Refused(Refusal::NotPermitted(TaskCreate))
}

fn reads_all_creates_in(paths: &[&str]) -> Vec<Grant> {
    vec![
        allowed(&[TaskRead], None),
        allowed(&[TaskCreate], Some(paths)),
    ]
}

#[test]
fn a_tokens_new_task_carries_the_parent_it_named() {
    let s = two_errands();
    let grants = vec![allowed(&[TaskCreate, TaskRead], None)];
    let Landed::Sent(sent) = token_new(&s, grants, milk("", Some(2))).unwrap() else {
        panic!("a token's task was written");
    };
    assert_eq!(sent.item.change.object().base().parent, Some(oid(2)));
}

#[test]
fn a_token_naming_a_parent_it_may_not_see_is_answered_as_if_it_were_missing() {
    let s = two_errands();
    let grants = vec![allowed(&[TaskCreate, TaskRead], Some(&["Work"]))];
    let err = token_new(&s, grants, milk("", Some(2))).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::NoSuchObject(oid(2).short().to_string()))
    );
}

#[test]
fn a_token_that_may_not_write_the_path_learns_nothing_of_the_namesakes() {
    let s = two_errands();
    let grants = vec![allowed(&[TaskCreate, TaskRead], Some(&["Work"]))];
    let err = token_new(&s, grants, milk("Home/errands", None)).unwrap_err();
    assert_eq!(err, not_permitted());
}

#[test]
fn a_token_that_sees_neither_namesake_is_refused_as_if_it_may_not_write_there() {
    let s = two_errands();
    let grants = vec![allowed(&[TaskCreate, TaskRead], Some(&["Home/errands"]))];
    let err = token_new(&s, grants, milk("Home/errands", None)).unwrap_err();
    assert_eq!(err, not_permitted());
    assert!(s.store.items().unwrap().is_empty());
}

#[test]
fn a_token_that_sees_one_namesake_is_refused_as_if_it_may_not_write_there() {
    let s = Scenario::default().drawing_from(9);
    s.filed(crate::testing::task_in(1, "errands", "Home"));
    let mut hidden = event(2);
    hidden.base_mut().subject = "errands".into();
    hidden.base_mut().path = Path::parse("Home").unwrap();
    s.working(hidden);
    let grants = vec![allowed(&[TaskCreate, TaskRead], None)];
    let err = token_new(&s, grants, milk("Home/errands", None)).unwrap_err();
    assert_eq!(err, not_permitted());
}

#[test]
fn a_token_that_sees_both_namesakes_is_asked_to_name_one() {
    let s = two_errands();
    let grants = vec![allowed(&[TaskCreate, TaskRead], None)];
    let err = token_new(&s, grants, milk("Home/errands", None)).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::SeveralHoldThePath {
            namesakes: vec![oid(1), oid(2)],
            folder: Path::parse("Home/errands").unwrap(),
            retry: Retry::New("buy milk".into()),
        })
    );
}

#[test]
fn a_token_that_may_not_write_there_is_refused_before_the_namesakes_are_counted() {
    let s = two_errands();
    let err = token_new(
        &s,
        reads_all_creates_in(&["Work"]),
        milk("Home/errands", None),
    );
    assert_eq!(err.unwrap_err(), not_permitted());
}

#[test]
fn a_token_that_may_not_write_there_is_refused_before_the_parents_folder_is_checked() {
    let s = two_errands();
    let err = token_new(
        &s,
        reads_all_creates_in(&["Work"]),
        milk("Elsewhere", Some(2)),
    );
    assert_eq!(err.unwrap_err(), not_permitted());
}

#[test]
fn a_token_that_may_not_write_there_is_refused_before_a_parent_it_cannot_see_is_looked_up() {
    let s = two_errands();
    let grants = vec![allowed(&[TaskCreate, TaskRead], Some(&["Work"]))];
    let err = token_new(&s, grants, milk("Home/errands", Some(2)));
    assert_eq!(err.unwrap_err(), not_permitted());
}

#[test]
fn a_tokens_new_task_with_an_id_is_refused_before_the_namesakes_are_counted() {
    let s = two_errands();
    let task = NewTask {
        id: Some(dam_domain::CallerId::parse("milk-1").unwrap()),
        ..milk("Home/errands", None)
    };
    let err = token_new(&s, reads_all_creates_in(&["Work"]), task);
    assert_eq!(err.unwrap_err(), not_permitted());
}

#[test]
fn a_token_that_sees_one_namesake_files_under_it_only_when_it_names_it() {
    let s = Scenario::default().drawing_from(9);
    s.filed(crate::testing::task_in(1, "errands", "Home"));
    let mut hidden = event(2);
    hidden.base_mut().subject = "errands".into();
    hidden.base_mut().path = Path::parse("Home").unwrap();
    s.working(hidden);
    let grants = vec![allowed(&[TaskCreate, TaskRead], None)];
    let Landed::Sent(sent) = token_new(&s, grants, milk("", Some(1))).unwrap() else {
        panic!("a token's task was written");
    };
    let filed = sent.item.change.object().base();
    assert_eq!(
        (filed.path.as_str(), filed.parent.clone()),
        ("Home/errands/", Some(oid(1)))
    );
}
