use dam_domain::{Object, Path};

use super::order::{held_upstream, sent};
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{Scenario, config, oid, task_in, todoist};

use MutationOp::{Create, Update};

pub(super) fn excepting_kitchen(s: Scenario) -> Scenario {
    let mut config = config();
    config.remotes[0].except = vec![Path::parse("kitchen").unwrap()];
    s.configured(config)
}

pub(super) fn committed_now(s: &Scenario) {
    s.dam().add_all().unwrap();
    s.dam().commit("m").unwrap();
}

#[test]
fn a_task_under_an_excepted_path_is_never_sent() {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "a", "kitchen/")))
        .committed(Object::Task(task_in(2, "b", "kitchen/shelf/")))
        .committed(Object::Task(task_in(3, "c", "Home/")));
    assert_eq!(sent(&s), vec![(oid(3), Create)]);
}

#[test]
fn the_task_that_holds_an_excepted_path_is_never_sent() {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "kitchen", "")))
        .committed(Object::Task(task_in(2, "kitchen", "")))
        .committed(Object::Task(task_in(3, "kitchenette", "")));
    assert_eq!(sent(&s), vec![(oid(3), Create)]);
}

#[test]
fn removing_the_task_that_holds_an_excepted_path_sends_no_delete() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "kitchen", "")]));
    s.deleted(&oid(1));
    assert_eq!(sent(&s), vec![]);
}

#[test]
fn a_task_the_remote_holds_is_left_as_it_is_once_its_path_is_excepted() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "a", "kitchen/")]));
    s.filed(task_in(1, "renamed", "kitchen/"));
    committed_now(&s);
    assert_eq!(sent(&s), vec![]);
    assert!(s.store.remote_id(&todoist(), &oid(1)).unwrap().is_some());
}

#[test]
fn a_task_moved_into_an_excepted_path_after_its_push_is_left_as_it_is() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "a", "Home/")]));
    s.filed(task_in(1, "a", "kitchen/"));
    committed_now(&s);
    assert_eq!(sent(&s), vec![]);
}

#[test]
fn removing_a_task_under_an_excepted_path_sends_no_delete() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "a", "kitchen/")]));
    s.deleted(&oid(1));
    assert_eq!(sent(&s), vec![]);
}

#[test]
fn removing_a_task_moved_into_an_excepted_path_sends_no_delete() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "a", "Home/")]));
    s.filed(task_in(1, "a", "kitchen/"));
    committed_now(&s);
    s.deleted(&oid(1));
    assert_eq!(sent(&s), vec![]);
}

#[test]
fn removing_a_task_moved_into_an_excepted_path_in_an_earlier_push_sends_no_delete() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "a", "Home/")]));
    s.filed(task_in(1, "a", "kitchen/"));
    committed_now(&s);
    assert_eq!(sent(&s), vec![]);
    s.deleted(&oid(1));
    assert_eq!(sent(&s), vec![]);
}

#[test]
fn an_update_to_a_task_outside_is_still_sent() {
    let s = excepting_kitchen(held_upstream(&[task_in(1, "a", "Home/")]));
    s.filed(task_in(1, "b", "Home/"));
    committed_now(&s);
    assert_eq!(sent(&s), vec![(oid(1), Update)]);
}

#[test]
fn a_task_left_out_is_counted_as_skipped_as_one_outside_path_is() {
    let s = excepting_kitchen(Scenario::default());
    s.committed(Object::Task(task_in(1, "a", "kitchen/")));
    let (l, _) = crate::testing::pushing(crate::testing::accepting_all(Some("i:new")));
    let reports = s
        .dam()
        .push(crate::testing::remotes_launched_by(&l), None)
        .unwrap();
    assert_eq!(reports[0].skipped, 1);
}
