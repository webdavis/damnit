use super::*;
use crate::errors::Retry;
use crate::ports::ObjectRepository;
use crate::testing::{Scenario, oid, task_in};

fn two_errands() -> Scenario {
    let s = Scenario::default();
    s.filed(task_in(5, "Home", ""))
        .filed(task_in(1, "errands", "Home"))
        .filed(task_in(2, "errands", "Home"))
        .filed(task_in(3, "post", "Work"))
        .filed(task_in(4, "stamps", "Work/post"));
    s
}

fn parent_of(s: &Scenario, byte: u8) -> Option<Oid> {
    s.store
        .get(&oid(byte))
        .unwrap()
        .unwrap()
        .base()
        .parent
        .clone()
}

#[test]
fn moving_under_a_parent_files_it_in_that_folder_and_carries_its_contents() {
    let s = two_errands();
    s.dam().relocate(&oid(3), None, Some(&oid(2))).unwrap();
    assert_eq!(s.path_of(&oid(3)), "Home/errands/");
    assert_eq!(parent_of(&s, 3), Some(oid(2)));
    assert_eq!(s.path_of(&oid(4)), "Home/errands/post/");
    assert_eq!(parent_of(&s, 4), Some(oid(3)));
}

#[test]
fn moving_under_something_it_holds_is_refused() {
    let s = two_errands();
    for target in [3, 4] {
        let err = s.dam().relocate(&oid(3), None, Some(&oid(target)));
        assert_eq!(
            err,
            Err(UseCaseError::Refused(Refusal::MoveInsideItself(oid(3))))
        );
    }
}

#[test]
fn moving_to_a_path_two_namesakes_hold_is_refused_until_one_is_named() {
    let s = two_errands();
    let to = Path::parse("Home/errands").unwrap();
    let err = s.dam().relocate(&oid(3), Some(&to), None).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::Refused(Refusal::SeveralHoldThePath {
            namesakes: vec![oid(1), oid(2)],
            folder: to,
            retry: Retry::Move(oid(3)),
        })
    );
    assert_eq!(s.path_of(&oid(3)), "Work/");
}

#[test]
fn a_token_is_refused_a_move_before_anything_about_the_place_is_checked() {
    let s = two_errands();
    let tool = crate::testing::holder(vec![crate::testing::allowed(
        &dam_domain::Permission::ALL,
        None,
    )]);
    let dam = Dam {
        access: &tool,
        ..s.dam()
    };
    let to = Path::parse("Home/errands").unwrap();
    assert_eq!(
        dam.relocate(&oid(3), Some(&to), None).unwrap_err(),
        UseCaseError::Refused(Refusal::NotOpenToTokens("mv"))
    );
}
