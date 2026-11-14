use crate::errors::UseCaseError;
use crate::ports::HelperError;
use crate::remote::{MutationOutcome, MutationVerdict, PushOutcome};
use crate::testing::prelude::*;
use crate::testing::{
    Scenario, ScriptedHelper, ScriptedLauncher, accepting_all, oid, pushing, remotes_launched_by,
    task, todoist,
};

fn answering_two_then(stop: HelperError) -> ScriptedLauncher {
    ScriptedLauncher::launching(move || {
        let stop = stop.clone();
        ScriptedHelper {
            push_answer: Box::new(move |ms| {
                Ok(PushOutcome {
                    outcomes: ms[..2]
                        .iter()
                        .map(|m| MutationOutcome {
                            oid: m.oid.clone(),
                            verdict: MutationVerdict::Accepted {
                                remote_id: Some(format!("r-{}", m.idempotency_key)),
                            },
                        })
                        .collect(),
                    stopped_by: Some(stop.clone()),
                })
            }),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    })
}

fn owed(s: &Scenario) -> Vec<dam_domain::Oid> {
    s.store
        .unpushed(&todoist())
        .unwrap()
        .iter()
        .flat_map(|r| r.changes.iter().map(|c| c.change.oid().clone()))
        .collect()
}

fn three_cut_short_by(stop: HelperError) {
    let s = Scenario::default();
    s.committed(task(1)).committed(task(2)).committed(task(3));

    let err = s
        .dam()
        .push(remotes_launched_by(&answering_two_then(stop.clone())), None)
        .unwrap_err();

    assert_eq!(
        err,
        UseCaseError::PushStopped {
            by: stop,
            kept: 2,
            unanswered: 1,
        }
    );
    for n in [1, 2] {
        assert!(
            s.store.remote_id(&todoist(), &oid(n)).unwrap().is_some(),
            "{n}"
        );
    }
    assert_eq!(s.store.remote_id(&todoist(), &oid(3)).unwrap(), None);
    assert_eq!(owed(&s), vec![oid(3)]);

    let (accepting, resent) = pushing(accepting_all(Some("r3")));
    s.dam().push(remotes_launched_by(&accepting), None).unwrap();
    let resent: Vec<_> = resent.borrow().iter().map(|m| m.oid.clone()).collect();
    assert_eq!(resent, vec![oid(3)]);
}

#[test]
fn a_timeout_after_two_answers_keeps_them_and_owes_the_third() {
    three_cut_short_by(HelperError::Timeout {
        helper: "todoist".into(),
        deadline: "1s".into(),
        said: None,
    });
}

#[test]
fn a_cancel_after_two_answers_keeps_them_and_owes_the_third() {
    three_cut_short_by(HelperError::Cancelled);
}
