use dam_domain::Oid;

use super::namesakes::why_held;
use super::refiles::{
    OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS, a_parent_replaced_by_a_namesake_over_a_child_it_held,
    pushed_to_a_remote_that_removes_only_empty_containers, refiled, sent,
};
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    accepting_all, oid, pushing, pushing_to_a_remote_that_removes_only_empty_containers,
    remotes_launched_by, todoist,
};

use MutationOp::{Create, Delete};

fn refusing(refused: Oid) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: if m.oid == refused {
                    MutationVerdict::Refused {
                        why: "temporary failure".into(),
                        unasked: false,
                    }
                } else {
                    MutationVerdict::Accepted {
                        remote_id: Some(format!("i:{}", m.oid.short())),
                    }
                },
            })
            .collect()
    }
}

#[test]
fn a_remote_that_does_not_decide_gets_the_refile_and_holds_the_old_parent_for_its_name() {
    for (old, new, child) in OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS {
        let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(old, new, child);
        let (l, pushed) = pushing(accepting_all(Some("i:new")));
        s.dam().push(remotes_launched_by(&l), None).unwrap();
        let sent = sent(&pushed);
        assert_eq!((sent[0].0.clone(), sent[0].1), (oid(new), Create));
        assert_eq!(sent[1..], [refiled(child)]);
        let why = why_held(&s, &oid(old));
        assert!(why.contains("same name in the same place"), "{why}");
    }
}

#[test]
fn a_refused_refile_holds_the_old_parent_until_a_later_push_refiles_the_child() {
    for (old, new, child) in OLD_NEW_AND_CHILD_IN_SEVERAL_OID_ORDERS {
        let s = a_parent_replaced_by_a_namesake_over_a_child_it_held(old, new, child);
        let (l, pushed) =
            pushing_to_a_remote_that_removes_only_empty_containers(refusing(oid(child)));
        s.dam().push(remotes_launched_by(&l), None).unwrap();
        let ops: Vec<MutationOp> = sent(&pushed).into_iter().map(|(_, op, ..)| op).collect();
        assert!(!ops.contains(&Delete), "{ops:?}");
        assert!(
            s.store
                .push_retries(&todoist())
                .unwrap()
                .contains(&oid(old))
        );
        assert!(s.store.remote_id(&todoist(), &oid(old)).unwrap().is_some());

        assert_eq!(
            sent(&pushed_to_a_remote_that_removes_only_empty_containers(&s)),
            vec![refiled(child), (oid(old), Delete, String::new(), vec![])]
        );
        assert!(s.store.push_retries(&todoist()).unwrap().is_empty());
        assert!(s.store.remote_id(&todoist(), &oid(old)).unwrap().is_none());
    }
}
