use dam_domain::{Field, Object, Oid, Task};

use super::order::held_upstream;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, accepting_all, oid, pushing_to_a_remote_that_removes_only_empty_containers,
    remotes_launched_by, task_caps, task_in, todoist,
};

use MutationOp::{Create, Delete, Update};

pub(super) const OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS: [(u8, u8, u8); 4] =
    [(1, 2, 3), (3, 2, 1), (2, 1, 3), (1, 3, 2)];

pub(super) type Sent = (Oid, MutationOp, String, Vec<Field>);

pub(super) fn sent(pushed: &Pushed) -> Vec<Sent> {
    pushed
        .borrow()
        .iter()
        .map(|m| {
            let path = m.object.as_ref().map(|o| o.base().path.to_string());
            (
                m.oid.clone(),
                m.op,
                path.unwrap_or_default(),
                m.fields.clone(),
            )
        })
        .collect()
}

pub(super) fn pushed_to_a_remote_that_removes_only_empty_containers(s: &Scenario) -> Pushed {
    let (l, pushed) =
        pushing_to_a_remote_that_removes_only_empty_containers(accepting_all(Some("i:new")));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    pushed
}

pub(super) fn a_parent_replaced_by_a_namesake_over_a_child_it_held(
    old: u8,
    new: u8,
    child: u8,
) -> Scenario {
    let s = held_upstream(&[task_in(old, "P", "Home/"), task_in(child, "c", "Home/P/")]);
    s.filed(task_in(new, "P", "Home/")).deleted(&oid(old));
    s
}

pub(super) fn refiled(child: u8) -> Sent {
    (oid(child), Update, "Home/P/".into(), vec![Field::Path])
}

fn assert_gone(s: &Scenario, removed: u8) {
    assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
    assert!(
        s.store
            .remote_id(&todoist(), &oid(removed))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_child_the_old_parent_held_is_refiled_under_its_new_namesake_in_one_push() {
    for (old, new, child) in OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS {
        let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(old, new, child);
        assert_eq!(
            sent(&pushed_to_a_remote_that_removes_only_empty_containers(&s)),
            vec![
                (oid(new), Create, "Home/".into(), task_caps().fields),
                refiled(child),
                (oid(old), Delete, String::new(), vec![]),
            ]
        );
        assert_gone(&s, old);
    }
}

#[test]
fn a_child_that_stays_beside_a_namesake_already_upstream_is_refiled() {
    for (removed, staying, child) in OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS {
        let s = held_upstream(&[
            task_in(removed, "P", "Home/"),
            task_in(staying, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.deleted(&oid(removed));
        assert_eq!(
            sent(&pushed_to_a_remote_that_removes_only_empty_containers(&s)),
            vec![
                refiled(child),
                (oid(removed), Delete, String::new(), vec![])
            ]
        );
        assert_gone(&s, removed);
    }
}

#[test]
fn an_owed_update_of_the_child_also_declares_its_place() {
    let (old, new, child) = (1, 2, 3);
    let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(old, new, child);
    let mut edited: Task = task_in(child, "c", "Home/P/");
    edited.base.body = "more".into();
    s.committed(Object::Task(edited));
    let pushed = sent(&pushed_to_a_remote_that_removes_only_empty_containers(&s));
    assert_eq!(pushed[1].3, vec![Field::Body, Field::Path]);
}

#[test]
fn a_task_deeper_in_the_folder_is_not_refiled() {
    let (old, new, child, deeper) = (1, 2, 3, 4);
    let s = held_upstream(&[
        task_in(old, "P", "Home/"),
        task_in(child, "c", "Home/P/"),
        task_in(deeper, "d", "Home/P/c/"),
    ]);
    s.filed(task_in(new, "P", "Home/")).deleted(&oid(old));
    let pushed = sent(&pushed_to_a_remote_that_removes_only_empty_containers(&s));
    let oids: Vec<Oid> = pushed.into_iter().map(|(o, ..)| o).collect();
    assert_eq!(oids, vec![oid(new), oid(child), oid(old)]);
    assert_gone(&s, old);
}

#[test]
fn a_child_removed_with_the_old_parent_is_not_refiled() {
    let (old, new, child) = (1, 2, 3);
    let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(old, new, child);
    s.deleted(&oid(child));
    let pushed = sent(&pushed_to_a_remote_that_removes_only_empty_containers(&s));
    let ops: Vec<(Oid, MutationOp)> = pushed.into_iter().map(|(o, op, ..)| (o, op)).collect();
    assert_eq!(
        ops,
        vec![(oid(new), Create), (oid(child), Delete), (oid(old), Delete)]
    );
}

fn containers_named(pushed: &Pushed) -> Vec<(Oid, Option<String>)> {
    pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.container_remote_id.clone()))
        .collect()
}

#[test]
fn a_refile_names_the_namesake_the_remote_already_holds() {
    for (removed, staying, child) in OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS {
        let s = held_upstream(&[
            task_in(removed, "P", "Home/"),
            task_in(staying, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.deleted(&oid(removed));
        assert_eq!(
            containers_named(&pushed_to_a_remote_that_removes_only_empty_containers(&s)),
            vec![(oid(child), Some("i:1".into())), (oid(removed), None)]
        );
    }
}

#[test]
fn a_refile_under_a_namesake_this_push_creates_names_no_container() {
    let (old, new, child) = (1, 2, 3);
    let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(old, new, child);
    let named = containers_named(&pushed_to_a_remote_that_removes_only_empty_containers(&s));
    assert!(
        named.iter().all(|(_, container)| container.is_none()),
        "{named:?}"
    );
}
