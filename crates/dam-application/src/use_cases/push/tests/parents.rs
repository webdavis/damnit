use dam_domain::{Object, Oid, Task};

use super::order::held_upstream;
use crate::remote::{MutationOp, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, accepting_all, oid, pushing, remotes_launched_by, task_in, todoist,
};
use crate::use_cases::edit::EditFields;

use MutationOp::{Create, Update};

pub(super) fn held_by(byte: u8, subject: &str, path: &str, parent: u8) -> Task {
    let mut t = task_in(byte, subject, path);
    t.base.parent = Some(oid(parent));
    t
}

pub(super) fn two_namesakes() -> Scenario {
    held_upstream(&[task_in(1, "S", "Home/"), task_in(2, "S", "Home/")])
}

fn pushed(s: &Scenario) -> (Pushed, Vec<(Oid, String)>) {
    let (l, pushed) = pushing(accepting_all(Some("i:new")));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    (pushed, report.failed)
}

fn named(pushed: &Pushed) -> Vec<(Oid, MutationOp, Option<String>)> {
    pushed
        .borrow()
        .iter()
        .map(|m: &RemoteMutation| (m.oid.clone(), m.op, m.container_remote_id.clone()))
        .collect()
}

#[test]
fn a_task_created_under_each_of_two_namesakes_names_its_own_by_remote_id() {
    let s = two_namesakes();
    s.committed(Object::Task(held_by(3, "a", "Home/S/", 1)))
        .committed(Object::Task(held_by(4, "b", "Home/S/", 2)));
    let (pushed, failed) = pushed(&s);
    assert!(failed.is_empty(), "{failed:?}");
    assert_eq!(
        named(&pushed),
        vec![
            (oid(3), Create, Some("i:0".into())),
            (oid(4), Create, Some("i:1".into())),
        ]
    );
}

#[test]
fn a_move_between_namesakes_names_the_new_parent_by_remote_id() {
    let s = held_upstream(&[
        task_in(1, "S", "Home/"),
        task_in(2, "S", "Home/"),
        held_by(3, "a", "Home/S/", 1),
    ]);
    s.committed(Object::Task(held_by(3, "a", "Home/S/", 2)));
    let (pushed, _) = pushed(&s);
    assert_eq!(named(&pushed), vec![(oid(3), Update, Some("i:1".into()))]);
}

#[test]
fn renaming_one_namesake_carries_only_its_own_task_into_it() {
    let s = held_upstream(&[
        task_in(1, "S", "Home/"),
        task_in(2, "S", "Home/"),
        held_by(3, "a", "Home/S/", 1),
        held_by(4, "b", "Home/S/", 2),
    ]);
    let renamed = EditFields {
        subject: Some("T".into()),
        ..EditFields::default()
    };
    s.dam().edit(&oid(2), renamed).unwrap();
    s.dam().add_all().unwrap();
    s.dam().commit("renamed").unwrap();
    let (pushed, _) = pushed(&s);
    assert_eq!(
        named(&pushed),
        vec![(oid(2), Update, None), (oid(4), Update, Some("i:1".into())),]
    );
}

#[test]
fn an_object_dam_cannot_place_between_two_namesakes_is_held_with_the_reason() {
    let s = two_namesakes();
    s.committed(Object::Task(task_in(3, "a", "Home/S/")));
    let (pushed, failed) = pushed(&s);
    assert!(pushed.borrow().is_empty(), "{:?}", named(&pushed));
    let (a, b, c) = (oid(1), oid(2), oid(3));
    let (a, b, c) = (a.short(), b.short(), c.short());
    assert_eq!(
        failed,
        vec![(
            oid(3),
            format!(
                "{c}: dam does not know whether {a} or {b} holds it; \
                 run: dam pull, or name it: dam mv {c} --parent {a}"
            )
        )]
    );
    assert!(s.store.push_retries(&todoist()).unwrap().contains(&oid(3)));
}

#[test]
fn a_task_under_a_parent_this_push_creates_names_that_parent_by_its_oid() {
    let s = two_namesakes();
    s.committed(Object::Task(task_in(5, "S", "Home/")))
        .committed(Object::Task(held_by(3, "a", "Home/S/", 5)));
    let (pushed, failed) = pushed(&s);
    assert!(failed.is_empty(), "{failed:?}");
    let named_by_oid: Vec<(Oid, Option<String>, Option<Oid>)> = pushed
        .borrow()
        .iter()
        .map(|m| {
            (
                m.oid.clone(),
                m.container_remote_id.clone(),
                m.container_oid.clone(),
            )
        })
        .collect();
    assert_eq!(
        named_by_oid,
        vec![(oid(5), None, None), (oid(3), None, Some(oid(5)))]
    );
}
