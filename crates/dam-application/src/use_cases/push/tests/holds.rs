use dam_domain::{Oid, Task};

use super::order::{BOTH_OID_ORDERS, held_upstream, sent};
use crate::ports::Notice;
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{Pushed, Scenario, oid, pushing, remotes_launched_by, task_in, todoist};

use MutationOp::{Delete, Update};

fn refusing(refused: Oid) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: if m.oid == refused {
                    MutationVerdict::Refused {
                        why: "temporary failure".into(),
                    }
                } else {
                    MutationVerdict::Accepted { remote_id: None }
                },
            })
            .collect()
    }
}

fn push_refusing(s: &Scenario, refused: u8) -> Pushed {
    let (l, pushed) = pushing(refusing(oid(refused)));
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

fn push_failed_why(s: &Scenario, held: u8) -> String {
    s.store
        .notices()
        .unwrap()
        .into_iter()
        .find_map(|n| match n {
            Notice::PushFailed { oid: o, why, .. } if o == oid(held) => Some(why),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no push failed notice for {held}"))
}

fn assert_held_for(s: &Scenario, held: u8, waits_on: u8) {
    let why = push_failed_why(s, held);
    assert!(why.contains(oid(waits_on).short()), "{why}");
    assert!(
        s.store
            .push_retries(&todoist())
            .unwrap()
            .contains(&oid(held))
    );
    assert!(s.store.remote_id(&todoist(), &oid(held)).unwrap().is_some());
}

struct Holder {
    container: Task,
    child_before: &'static str,
    child_after: &'static str,
}

fn holders(container: u8) -> [Holder; 3] {
    [
        Holder {
            container: task_in(container, "P", "Home/"),
            child_before: "Home/P/",
            child_after: "Home/",
        },
        Holder {
            container: task_in(container, "S", "Home/"),
            child_before: "Home/S/",
            child_after: "Home/",
        },
        Holder {
            container: task_in(container, "Home", ""),
            child_before: "Home/",
            child_after: "Work/",
        },
    ]
}

#[test]
fn a_container_is_not_removed_while_the_move_out_of_it_is_refused() {
    for (container, child) in BOTH_OID_ORDERS {
        for holder in holders(container) {
            let s = held_upstream(&[holder.container, task_in(child, "c", holder.child_before)]);
            s.filed(task_in(child, "c", holder.child_after))
                .deleted(&oid(container));
            let pushed = push_refusing(&s, child);
            assert_eq!(ops(&pushed), vec![(oid(child), Update)]);
            assert_held_for(&s, container, child);
            assert!(
                s.store
                    .push_retries(&todoist())
                    .unwrap()
                    .contains(&oid(child))
            );
            assert_eq!(
                sent(&s),
                vec![(oid(child), Update), (oid(container), Delete)]
            );
        }
    }
}

#[test]
fn an_unrelated_refusal_does_not_hold_a_removal() {
    for (parent, unrelated) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(unrelated, "u", "Work/"),
        ]);
        s.filed(task_in(unrelated, "renamed", "Work/"))
            .deleted(&oid(parent));
        let pushed = push_refusing(&s, unrelated);
        assert!(ops(&pushed).contains(&(oid(parent), Delete)));
        assert!(
            s.store
                .remote_id(&todoist(), &oid(parent))
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn a_refused_inner_removal_holds_the_outer_one() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.deleted(&oid(child)).deleted(&oid(parent));
        let pushed = push_refusing(&s, child);
        assert_eq!(ops(&pushed), vec![(oid(child), Delete)]);
        assert_held_for(&s, parent, child);
        assert_eq!(sent(&s), vec![(oid(child), Delete), (oid(parent), Delete)]);
    }
}

#[test]
fn every_planned_change_ends_accepted_refused_or_held() {
    let (parent, child, unrelated) = (1, 2, 3);
    let s = held_upstream(&[
        task_in(parent, "P", "Home/"),
        task_in(child, "c", "Home/P/"),
        task_in(unrelated, "u", "Work/"),
    ]);
    s.filed(task_in(unrelated, "renamed", "Work/"))
        .deleted(&oid(child))
        .deleted(&oid(parent));
    let (l, _) = pushing(refusing(oid(child)));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    let accepted = report.succeeded;
    let refused_or_held: Vec<Oid> = report.failed.into_iter().map(|(oid, _)| oid).collect();
    assert_eq!((report.sent, accepted), (2, 1));
    assert_eq!(refused_or_held, vec![oid(child), oid(parent)]);
    assert_eq!(s.store.push_retries(&todoist()).unwrap(), refused_or_held);
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}
