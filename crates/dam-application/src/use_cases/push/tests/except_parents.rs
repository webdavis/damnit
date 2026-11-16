use dam_domain::{Object, Oid};

use super::except::excepting_kitchen;
use super::order::{held_upstream, sent};
use super::parents::held_by;
use crate::ports::Notice;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, task_in, todoist};

use MutationOp::Create;

fn why_held(s: &Scenario, held: &Oid) -> String {
    s.store
        .notices()
        .unwrap()
        .into_iter()
        .find_map(|n| match n {
            Notice::PushFailed { oid, why, .. } if &oid == held => Some(why),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn a_child_under_an_excepted_path_stays_off_the_remote_beside_its_sent_parent() {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "P", "Home/")))
        .committed(Object::Task(held_by(2, "c", "kitchen/", 1)));
    assert_eq!(sent(&s), vec![(oid(1), Create)]);
}

#[test]
fn a_child_outside_whose_parent_is_excepted_is_held_not_filed_elsewhere() {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "P", "kitchen/")))
        .committed(Object::Task(held_by(2, "c", "Home/", 1)));
    assert_eq!(sent(&s), vec![]);
    let why = why_held(&s, &oid(2));
    assert!(why.contains("remote.todoist.except"), "{why}");
}

#[test]
fn a_child_outside_whose_parent_holds_the_excepted_path_is_held() {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "kitchen", "")))
        .committed(Object::Task(held_by(2, "c", "Home/", 1)));
    assert_eq!(sent(&s), vec![]);
    let why = why_held(&s, &oid(2));
    assert!(why.contains("remote.todoist.except"), "{why}");
}

#[test]
fn a_child_outside_names_its_excepted_parent_the_remote_already_holds() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "P", "kitchen/")]));
    s.committed(Object::Task(held_by(2, "c", "Home/", 1)));
    assert_eq!(sent(&s), vec![(oid(2), Create)]);
}

#[test]
fn removing_a_parent_the_remote_holds_waits_on_its_excepted_child_upstream() {
    let s = excepting_kitchen(held_upstream(&[
        task_in(1, "P", "Home/"),
        held_by(2, "c", "kitchen/", 1),
    ]));
    s.deleted(&oid(1));
    assert_eq!(sent(&s), vec![]);
    assert!(s.store.push_retries(&todoist()).unwrap().contains(&oid(1)));
    let why = why_held(&s, &oid(1));
    assert!(why.contains(oid(2).short()), "{why}");
    assert!(
        why.contains(&format!(
            "run: dam mv {} PATH outside remote.todoist.except, then dam push",
            oid(2).short()
        )),
        "{why}"
    );
}
