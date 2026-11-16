use dam_domain::{Object, Oid};

use super::order::held_upstream;
use super::parents::{held_by, two_namesakes};
use crate::ports::Notice;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, accepting_all, oid, pushing, remotes_launched_by, task_in, todoist,
};
use crate::use_cases::prefix::Named;

use MutationOp::{Create, Delete, Update};

fn pushed_once(s: &Scenario) -> Pushed {
    let (l, pushed) = pushing(accepting_all(Some("i:new")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed
}

fn ops(pushed: &Pushed) -> Vec<(Oid, MutationOp)> {
    pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect()
}

pub(super) fn why_held(s: &Scenario, held: &Oid) -> String {
    assert!(s.store.push_retries(&todoist()).unwrap().contains(held));
    assert!(s.store.remote_id(&todoist(), held).unwrap().is_some());
    s.store
        .notices()
        .unwrap()
        .into_iter()
        .rev()
        .find_map(|n| match n {
            Notice::PushFailed { oid: o, why, .. } if &o == held => Some(why),
            _ => None,
        })
        .expect("a push failed notice for the removal")
}

#[test]
fn removing_one_of_two_namesakes_leaves_the_others_task_in_place_upstream() {
    let s = held_upstream(&[
        task_in(1, "S", "Home/"),
        task_in(2, "S", "Home/"),
        held_by(3, "a", "Home/S/", 1),
        held_by(4, "b", "Home/S/", 2),
    ]);
    s.dam().remove(&oid(1)).unwrap();
    s.dam().add_all().unwrap();
    s.dam().commit("removed").unwrap();
    assert_eq!(
        ops(&pushed_once(&s)),
        vec![(oid(3), Update), (oid(1), Delete)]
    );
    assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
}

#[test]
fn a_parent_replaced_by_a_namesake_is_removed_beside_it() {
    let s = held_upstream(&[task_in(1, "P", "Home/")]);
    s.deleted(&oid(1))
        .filed(task_in(2, "P", "Home/"))
        .committed(Object::Task(task_in(3, "c", "Home/P/")));
    assert_eq!(
        ops(&pushed_once(&s)),
        vec![(oid(2), Create), (oid(3), Create), (oid(1), Delete)]
    );
    assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
}

#[test]
fn a_removal_waits_on_what_it_still_holds_upstream() {
    let s = held_upstream(&[task_in(1, "P", "Home/"), task_in(2, "c", "Home/P/")]);
    s.deleted(&oid(1));
    assert!(ops(&pushed_once(&s)).is_empty());
    let c = oid(2);
    let c = c.short();
    assert_eq!(
        why_held(&s, &oid(1)),
        format!(
            "{c} is still under it upstream and this push does not move it; \
             run: dam mv {c} PATH, then dam push"
        )
    );
}

#[test]
fn a_removal_whose_contents_move_out_uncommitted_says_to_commit_them() {
    let s = held_upstream(&[task_in(1, "P", "Home/"), task_in(2, "c", "Home/P/")]);
    s.dam().remove(&oid(1)).unwrap();
    s.dam().add(&[Named::Object(oid(1))]).unwrap();
    s.dam().commit("removed").unwrap();
    pushed_once(&s);
    let c = oid(2);
    let c = c.short();
    assert_eq!(
        why_held(&s, &oid(1)),
        format!(
            "{c} is still under it upstream, and its move out is not committed; \
             run: dam add {c}, then dam commit and dam push"
        )
    );
}

#[test]
fn a_removal_waits_on_a_task_that_may_be_held_by_either_namesake_upstream() {
    let s = two_namesakes();
    s.committed(Object::Task(task_in(3, "c", "Home/S/")))
        .tracked_as(&oid(3), "i:3");
    s.pushed();
    s.deleted(&oid(1));
    pushed_once(&s);
    let (a, b, c) = (oid(1), oid(2), oid(3));
    let (a, b, c) = (a.short(), b.short(), c.short());
    assert_eq!(
        why_held(&s, &oid(1)),
        format!("{c} may be held by {a} or {b} upstream; run: dam pull, then dam push")
    );
}
