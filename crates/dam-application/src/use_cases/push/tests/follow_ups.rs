use dam_domain::{Field, Object, Task};

use crate::errors::UseCaseError;
use crate::ports::{AttemptRepository, HelperError};
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, PushOutcome, RemoteMutation};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, oid, pushing, remotes_launched_by,
    task, todoist,
};

fn created_with_no_answer_then_edited() -> Scenario {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    s.committed(Object::Task(Task::new(oid(1), "edited")));
    s
}

fn written(sent: &[RemoteMutation]) -> Vec<(MutationOp, Vec<Field>)> {
    sent.iter().map(|m| (m.op, m.fields.clone())).collect()
}

fn accepting_creates_only(ms: &[RemoteMutation]) -> Vec<MutationOutcome> {
    let creates = ms.iter().filter(|m| m.op == MutationOp::Create);
    creates
        .map(|m| MutationOutcome {
            oid: m.oid.clone(),
            verdict: MutationVerdict::Accepted {
                remote_id: Some("r1".into()),
            },
        })
        .collect()
}

#[test]
fn a_create_accepted_after_an_attempt_that_differs_is_followed_by_an_update_of_what_differs() {
    let s = created_with_no_answer_then_edited();
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    let report = s
        .dam()
        .push(remotes_launched_by(&l), None)
        .unwrap()
        .remove(0);
    let sent = pushed.borrow();
    assert_eq!(
        written(&sent)[1..],
        [(MutationOp::Update, vec![Field::Subject])]
    );
    assert_eq!(sent[0].op, MutationOp::Create);
    assert_eq!(sent[1].remote_id.as_deref(), Some("r1"));
    assert_ne!(sent[0].idempotency_key, sent[1].idempotency_key);
    assert_eq!((report.sent, report.succeeded), (2, 2));
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_create_accepted_after_one_attempt_and_no_edit_is_not_followed_up() {
    let s = Scenario::default();
    s.committed(task(1));
    let (l, _) = pushing(|_| vec![]);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (l, pushed) = pushing(accepting_all(Some("r1")));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let ops: Vec<MutationOp> = pushed.borrow().iter().map(|m| m.op).collect();
    assert_eq!(ops, vec![MutationOp::Create]);
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_follow_up_with_no_answer_is_sent_again_under_its_key() {
    let s = created_with_no_answer_then_edited();
    let (l, first) = pushing(accepting_creates_only);
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (l, again) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    let (first, again) = (first.borrow(), again.borrow());
    assert_eq!(
        written(&again),
        [(MutationOp::Update, vec![Field::Subject])]
    );
    assert_eq!(again[0].idempotency_key, first[1].idempotency_key);
    assert!(s.store.attempts(&todoist()).unwrap().is_empty());
}

#[test]
fn a_follow_up_stopped_before_any_answer_keeps_the_first_round_s_report() {
    let s = created_with_no_answer_then_edited();
    let calls = std::cell::Cell::new(0);
    let l = ScriptedLauncher::launching(move || {
        let calls = calls.clone();
        ScriptedHelper {
            push_answer: Box::new(move |ms| {
                calls.set(calls.get() + 1);
                match calls.get() {
                    1 => Ok(PushOutcome {
                        outcomes: accepting_creates_only(ms),
                        stopped_by: None,
                    }),
                    _ => Err(HelperError::Cancelled),
                }
            }),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    });
    let err = s.dam().push(remotes_launched_by(&l), None).unwrap_err();
    assert_eq!(
        err,
        UseCaseError::PushStopped {
            by: HelperError::Cancelled,
            kept: 1,
            unanswered: 1,
        }
    );
    assert_eq!(
        s.store.remote_id(&todoist(), &oid(1)).unwrap().as_deref(),
        Some("r1")
    );
    let (l, next) = pushing(accepting_all(None));
    s.dam().push(remotes_launched_by(&l), None).unwrap();
    assert_eq!(
        written(&next.borrow()),
        [(MutationOp::Update, vec![Field::Subject])]
    );
}
