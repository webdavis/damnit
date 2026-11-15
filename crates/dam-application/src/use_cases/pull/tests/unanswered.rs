use dam_domain::{Object, Task};

use crate::remote::{
    IncomingObject, MutationOp, MutationOutcome, MutationVerdict, PullOutcome, RemoteMutation,
};
use crate::testing::prelude::*;
use crate::testing::{Scenario, accepting_all, oid, pulling, pushing, remotes_launched_by};

pub(super) type Sent = Vec<(MutationOp, Option<String>, Option<String>)>;

pub(super) fn named(subject: &str) -> Object {
    Object::Task(Task::new(oid(1), subject))
}

pub(super) fn pulled(object: Object, remote_id: &str) -> IncomingObject {
    IncomingObject {
        remote_id: remote_id.into(),
        object,
    }
}

pub(super) fn push_with_no_answer(s: &Scenario) {
    let (l, _) = pushing(|_| vec![]);
    let _ = s.dam().push(remotes_launched_by(&l), None);
}

pub(super) fn push_answering_creates_only(s: &Scenario, remote_id: &str) {
    let remote_id = remote_id.to_string();
    let (l, _) = pushing(move |ms: &[RemoteMutation]| {
        ms.iter()
            .filter(|m| m.op == MutationOp::Create)
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                verdict: MutationVerdict::Accepted {
                    remote_id: Some(remote_id.clone()),
                },
            })
            .collect()
    });
    let _ = s.dam().push(remotes_launched_by(&l), None);
}

pub(super) fn pull(s: &Scenario, objects: Vec<IncomingObject>) {
    pull_outcome(
        s,
        PullOutcome {
            objects,
            ..PullOutcome::default()
        },
    );
}

pub(super) fn pull_outcome(s: &Scenario, outcome: PullOutcome) {
    s.dam()
        .pull(remotes_launched_by(&pulling(outcome)), None)
        .unwrap();
}

pub(super) fn subject(s: &Scenario) -> String {
    s.store
        .get(&oid(1))
        .unwrap()
        .unwrap()
        .base()
        .subject
        .clone()
}

pub(super) fn push_accepting(s: &Scenario, remote_id: &str) -> Sent {
    let (l, pushed) = pushing(accepting_all(Some(remote_id)));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let sent = pushed.borrow();
    sent.iter()
        .map(|m| {
            let subject = m.object.as_ref().map(|o| o.base().subject.clone());
            (m.op, m.remote_id.clone(), subject)
        })
        .collect()
}
