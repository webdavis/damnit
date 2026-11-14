use dam_domain::Oid;

use super::order::{BOTH_OID_ORDERS, held_upstream};
use crate::ports::Notice;
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{Scenario, oid, pushing, remotes_launched_by, task, task_in, todoist};

const REASON: &str = "Todoist did not answer in time";

fn unanswered(stuck: Oid) -> impl Fn(&[RemoteMutation]) -> Vec<MutationOutcome> + Clone + 'static {
    move |ms| {
        ms.iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: match m.oid == stuck {
                    true => MutationVerdict::Unanswered { why: REASON.into() },
                    false => MutationVerdict::Accepted {
                        remote_id: Some("i:new".into()),
                    },
                },
            })
            .collect()
    }
}

fn push_failed_why(s: &Scenario, about: u8) -> Vec<String> {
    s.store
        .notices()
        .unwrap()
        .into_iter()
        .filter_map(|n| match n {
            Notice::PushFailed { oid: o, why, .. } if o == oid(about) => Some(why),
            _ => None,
        })
        .collect()
}

#[test]
fn an_unanswered_verdict_keeps_its_commit_unpushed_and_says_why() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    let (l, pushed) = pushing(unanswered(oid(1)));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    assert_eq!(report.succeeded, 1);
    assert_eq!(report.failed, vec![(oid(1), REASON.to_string())]);
    assert_eq!(push_failed_why(&s, 1), vec![REASON.to_string()]);
    let still_unpushed = s.store.unpushed(&todoist()).unwrap();
    assert_eq!(still_unpushed.len(), 1);
    assert!(
        still_unpushed[0]
            .changes
            .iter()
            .any(|c| *c.change.oid() == oid(1))
    );

    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    let keys_of_1: Vec<&str> = sent
        .iter()
        .filter(|m| m.oid == oid(1))
        .map(|m| m.idempotency_key.as_str())
        .collect();
    assert_eq!(keys_of_1.len(), 2, "{sent:?}");
    assert_eq!(keys_of_1[0], keys_of_1[1], "the resend minted a fresh key");
    assert!(
        sent.iter()
            .filter(|m| m.oid == oid(1))
            .all(|m| m.op == MutationOp::Create)
    );
}

#[test]
fn a_removal_that_waits_on_an_unanswered_move_is_held() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.filed(task_in(child, "c", "Home/")).deleted(&oid(parent));
        let (l, pushed) = pushing(unanswered(oid(child)));
        s.dam().push(remotes_launched_by(&l), None).unwrap();
        let ops: Vec<(Oid, MutationOp)> = pushed
            .borrow()
            .iter()
            .map(|m| (m.oid.clone(), m.op))
            .collect();
        assert_eq!(ops, vec![(oid(child), MutationOp::Update)]);
        let held = push_failed_why(&s, parent);
        assert!(
            held.iter().any(|why| why.contains(oid(child).short())),
            "{held:?}"
        );
        assert!(
            s.store
                .remote_id(&todoist(), &oid(parent))
                .unwrap()
                .is_some()
        );
        assert_eq!(push_failed_why(&s, child), vec![REASON.to_string()]);
    }
}
