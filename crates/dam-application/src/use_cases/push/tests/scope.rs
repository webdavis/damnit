use dam_domain::{Oid, Path};

use super::order::{BOTH_OID_ORDERS, held_upstream};
use crate::ports::Notice;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, accepting_all, config, oid, pushing, remotes_launched_by, task_in, todoist,
};

use MutationOp::{Delete, Update};

fn scoped_to_home(s: Scenario) -> Scenario {
    let mut config = config();
    config.remotes[0].path = Some(Path::parse("Home").unwrap());
    s.configured(config)
}

fn push(s: &Scenario) -> Vec<(Oid, MutationOp)> {
    let (l, pushed) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed
        .borrow()
        .iter()
        .map(|m| (m.oid.clone(), m.op))
        .collect()
}

fn push_failed_why(s: &Scenario, held: &Oid) -> String {
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

fn a_child_moved_out_of_scope(parent: u8, child: u8) -> Scenario {
    let s = scoped_to_home(held_upstream(&[
        task_in(parent, "P", "Home/"),
        task_in(child, "c", "Home/P/"),
    ]));
    s.filed(task_in(child, "c", "Work/"));
    s
}

fn assert_held_until_moved_back(s: &Scenario, parent: u8, child: u8) {
    assert!(
        s.store
            .remote_id(&todoist(), &oid(parent))
            .unwrap()
            .is_some()
    );
    let why = push_failed_why(s, &oid(parent));
    assert!(why.contains(oid(child).short()), "{why}");
    assert!(why.contains("inside this remote's path"), "{why}");
    s.filed(task_in(child, "c", "Home/"));
    s.dam().add_all().unwrap();
    s.dam().commit("back in scope").unwrap();
    assert_eq!(push(s), vec![(oid(child), Update), (oid(parent), Delete)]);
}

#[test]
fn a_removal_waits_on_a_child_moved_out_of_scope_in_the_same_push() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = a_child_moved_out_of_scope(parent, child);
        s.deleted(&oid(parent));
        assert_eq!(push(&s), vec![]);
        assert_held_until_moved_back(&s, parent, child);
    }
}

#[test]
fn a_removal_waits_on_a_child_moved_out_of_scope_in_an_earlier_push() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = a_child_moved_out_of_scope(parent, child);
        s.dam().add_all().unwrap();
        s.dam().commit("out of scope").unwrap();
        assert_eq!(push(&s), vec![]);
        s.deleted(&oid(parent));
        assert_eq!(push(&s), vec![]);
        assert_held_until_moved_back(&s, parent, child);
        assert_eq!(push(&s), vec![]);
    }
}

#[test]
fn an_outer_removal_is_held_when_the_inner_one_waits_on_a_child_out_of_scope() {
    for (parent, inner, child) in [(1, 2, 3), (3, 2, 1), (2, 3, 1)] {
        let s = scoped_to_home(held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(inner, "C", "Home/P/"),
            task_in(child, "x", "Home/P/C/"),
        ]));
        s.filed(task_in(child, "x", "Work/"))
            .deleted(&oid(inner))
            .deleted(&oid(parent));
        assert_eq!(push(&s), vec![]);
        let retries = s.store.push_retries(&todoist()).unwrap();
        for held in [parent, inner] {
            assert!(!push_failed_why(&s, &oid(held)).is_empty());
            assert!(retries.contains(&oid(held)));
        }
        s.filed(task_in(child, "x", "Home/"));
        s.dam().add_all().unwrap();
        s.dam().commit("back in scope").unwrap();
        assert_eq!(
            push(&s),
            vec![
                (oid(child), Update),
                (oid(inner), Delete),
                (oid(parent), Delete)
            ]
        );
    }
}
