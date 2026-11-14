use crate::errors::UseCaseError;
use crate::ports::HelperError;
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, PushOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, oid, pushing,
    remotes_launched_by, task, todoist,
};

fn rate_limited() -> HelperError {
    HelperError::Remote("todoist is rate limiting this account; retry after 42s".into())
}

fn stopping_after(first: MutationVerdict) -> (ScriptedLauncher, Pushed) {
    let pushed = Pushed::default();
    let seen = pushed.clone();
    let launcher = ScriptedLauncher::launching(move || {
        let first = first.clone();
        ScriptedHelper {
            push_answer: Box::new(move |ms| {
                Ok(PushOutcome {
                    outcomes: vec![MutationOutcome {
                        oid: ms[0].oid.clone(),
                        verdict: first.clone(),
                    }],
                    stopped_by: Some(rate_limited()),
                })
            }),
            pushed: seen.clone(),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    });
    (launcher, pushed)
}

fn answering_the_first_then_stopping() -> (ScriptedLauncher, Pushed) {
    stopping_after(MutationVerdict::Accepted {
        remote_id: Some("r1".into()),
    })
}

#[test]
fn a_stop_on_the_first_call_keeps_what_the_helper_answered_before_it() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2)).committed(task(3));
    let (stopping, first) = answering_the_first_then_stopping();

    let err = s
        .dam()
        .push(remotes_launched_by(&stopping), None)
        .unwrap_err();

    assert_eq!(
        err,
        UseCaseError::PushStopped {
            by: rate_limited(),
            kept: 1,
            unanswered: 2,
        }
    );
    assert_eq!(first.borrow()[0].oid, oid(1));
    let mapped = s.store.remote_id(&todoist(), &oid(1)).unwrap();
    assert_eq!(mapped.as_deref(), Some("r1"));
    assert!(
        s.store
            .remote_snapshot(&todoist(), &oid(1))
            .unwrap()
            .is_some()
    );
    let owed: Vec<_> = s
        .store
        .unpushed(&todoist())
        .unwrap()
        .iter()
        .flat_map(|r| r.changes.iter().map(|c| c.change.oid().clone()))
        .collect();
    assert_eq!(owed, vec![oid(2), oid(3)]);

    let (accepting, second) = pushing(accepting_all(Some("r2")));
    s.dam().push(remotes_launched_by(&accepting), None).unwrap();

    let resent: Vec<_> = second
        .borrow()
        .iter()
        .filter(|m| m.op == MutationOp::Create)
        .map(|m| (m.oid.clone(), m.idempotency_key.clone()))
        .collect();
    let first_keys: Vec<_> = first.borrow()[1..]
        .iter()
        .map(|m| (m.oid.clone(), m.idempotency_key.clone()))
        .collect();
    assert_eq!(
        resent, first_keys,
        "only what is still owed, under the same keys"
    );
    assert!(s.store.unpushed(&todoist()).unwrap().is_empty());
}

#[test]
fn a_stop_before_any_verdict_stores_nothing() {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2));
    let (stopping, _) = stopping_after(MutationVerdict::Unanswered {
        why: "todoist is rate limiting this account; retry after 42s".into(),
    });

    let err = s
        .dam()
        .push(remotes_launched_by(&stopping), None)
        .unwrap_err();

    assert_eq!(err, UseCaseError::Helper(rate_limited()));
    assert!(s.store.notices().unwrap().is_empty());
    assert_eq!(s.store.last_push(&todoist()).unwrap(), None);
    assert_eq!(s.store.unpushed(&todoist()).unwrap().len(), 2);
}
