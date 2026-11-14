use dam_domain::{Object, Oid};

use super::order::held_upstream;
use crate::ports::Notice;
use crate::remote::MutationOp;
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, accepting_all, oid, pushing,
    pushing_to_a_remote_that_removes_only_empty_containers, remotes_launched_by, task_in, todoist,
};
use crate::use_cases::edit::EditFields;

use MutationOp::{Create, Delete, Update};

const OLD_NEW_AND_CHILD_IN_EVERY_OID_ORDER: [(u8, u8, u8); 3] = [(1, 2, 3), (3, 2, 1), (2, 1, 3)];

fn a_parent_replaced_by_a_namesake_with_a_child(old: u8, new: u8, child: u8) -> Scenario {
    let s = held_upstream(&[task_in(old, "P", "Home/")]);
    s.deleted(&oid(old))
        .filed(task_in(new, "P", "Home/"))
        .committed(Object::Task(task_in(child, "c", "Home/P/")));
    s
}

fn pushed_once(s: &Scenario) -> Pushed {
    let (l, pushed) = pushing(accepting_all(Some("i:new")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    pushed
}

fn sent_with_paths(pushed: &Pushed) -> Vec<(Oid, MutationOp, String)> {
    pushed
        .borrow()
        .iter()
        .map(|m| {
            let path = m
                .object
                .as_ref()
                .map(|o| o.base().path.as_str().to_string());
            (m.oid.clone(), m.op, path.unwrap_or_default())
        })
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
        .expect("a push failed notice for the old parent")
}

#[test]
fn a_parent_replaced_by_a_namesake_is_held_until_the_namesake_is_renamed() {
    for (old, new, child) in OLD_NEW_AND_CHILD_IN_EVERY_OID_ORDER {
        let s = a_parent_replaced_by_a_namesake_with_a_child(old, new, child);
        assert_eq!(
            sent_with_paths(&pushed_once(&s)),
            vec![
                (oid(new), Create, "Home/".to_string()),
                (oid(child), Create, "Home/P/".to_string()),
            ]
        );
        let why = why_held(&s, &oid(old));
        assert!(why.contains("same name in the same place"), "{why}");
        assert!(why.contains(oid(child).short()), "{why}");
        assert!(
            why.contains(&format!("rename {}, then push", oid(new).short())),
            "{why}"
        );
        let renamed = EditFields {
            subject: Some("Q".into()),
            ..EditFields::default()
        };
        s.dam().edit(&oid(new), renamed).unwrap();
        s.dam().add_all().unwrap();
        s.dam().commit("renamed").unwrap();
        assert_eq!(
            sent_with_paths(&pushed_once(&s)),
            vec![
                (oid(new), Update, "Home/".to_string()),
                (oid(child), Update, "Home/Q/".to_string()),
                (oid(old), Delete, String::new()),
            ]
        );
        assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
        assert!(s.store.remote_id(&todoist(), &oid(old)).unwrap().is_none());
    }
}

#[test]
fn a_remote_that_removes_only_empty_containers_gets_the_old_parent_removal_after_the_child() {
    for (old, new, child) in OLD_NEW_AND_CHILD_IN_EVERY_OID_ORDER {
        let s = a_parent_replaced_by_a_namesake_with_a_child(old, new, child);
        let (l, pushed) =
            pushing_to_a_remote_that_removes_only_empty_containers(accepting_all(Some("i:new")));
        let report = s
            .dam()
            .push(remotes_launched_by(&l), None)
            .unwrap()
            .remove(0);
        assert_eq!(
            sent_with_paths(&pushed),
            vec![
                (oid(new), Create, "Home/".to_string()),
                (oid(child), Create, "Home/P/".to_string()),
                (oid(old), Delete, String::new()),
            ]
        );
        assert!(report.failed.is_empty(), "{:?}", report.failed);
        assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
        assert!(s.store.remote_id(&todoist(), &oid(old)).unwrap().is_none());
    }
}
