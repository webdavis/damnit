use dam_domain::{Force, Path, Tracked};

use super::*;
use crate::errors::{Refusal, UseCaseError};
use crate::testing::prelude::ObjectRepository;

fn untracked(_: &Oid) -> Tracked {
    Tracked::No
}

#[test]
fn done_completes_every_object_in_the_set() {
    let s = work_and_home();
    let out = s
        .dam()
        .complete_each(&query("path:work/"), Force::No, &untracked, run())
        .unwrap();
    assert_eq!(out.changes.len(), 2);
    for byte in [1, 2] {
        let object = s.store.get(&oid(byte)).unwrap().unwrap();
        assert!(object.as_task().unwrap().completion.is_done());
    }
}

#[test]
fn done_on_a_set_holding_an_event_changes_nothing() {
    let s = work_and_home();
    s.working(crate::testing::event(4));
    let before = s.store.clone();
    assert_eq!(
        s.dam()
            .complete_each(&named(&[1, 4]), Force::No, &untracked, run()),
        Err(UseCaseError::Refused(Refusal::NotATask(oid(4))))
    );
    assert_eq!(s.store, before);
}

#[test]
fn mv_moves_each_object_and_a_child_moves_with_its_parent() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", "a"))
        .filed(task_in(2, "step", "a/plan"))
        .filed(task_in(3, "other", "c"));
    let to = Path::parse("b").unwrap();
    let out = s
        .dam()
        .relocate_each(&named(&[2, 1, 3]), Some(&to), None, run())
        .unwrap();
    assert_eq!(s.path_of(&oid(1)), "b/");
    assert_eq!(s.path_of(&oid(2)), "b/plan/");
    assert_eq!(s.path_of(&oid(3)), "b/");
    assert_eq!(out.changes.len(), 3);
}

#[test]
fn mv_into_a_member_of_the_set_changes_nothing() {
    let s = Scenario::default();
    s.filed(task_in(1, "plan", "a"))
        .filed(task_in(3, "other", "c"));
    let before = s.store.clone();
    let into_itself = Path::parse("a/plan/x").unwrap();
    assert!(
        s.dam()
            .relocate_each(&named(&[3, 1]), Some(&into_itself), None, run())
            .is_err()
    );
    assert_eq!(s.store, before);
}

#[test]
fn rm_removes_every_object_in_the_set() {
    let s = work_and_home();
    let out = s.dam().remove_each(&query("path:work/"), run()).unwrap();
    assert_eq!(out.changes.len(), 2);
    assert!(s.store.get(&oid(1)).unwrap().is_none());
    assert!(s.store.get(&oid(2)).unwrap().is_none());
    assert!(s.store.get(&oid(3)).unwrap().is_some());
}
