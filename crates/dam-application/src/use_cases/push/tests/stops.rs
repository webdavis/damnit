use std::cell::Cell;
use std::rc::Rc;

use super::order::{BOTH_OID_ORDERS, held_upstream, sent};
use crate::errors::UseCaseError;
use crate::ports::HelperError;
use crate::remote::{MutationOp, MutationOutcome, MutationVerdict};
use crate::testing::prelude::*;
use crate::testing::{
    Pushed, ScriptedHelper, ScriptedLauncher, oid, remotes_launched_by, task_in, todoist,
};

use MutationOp::{Delete, Update};

fn rate_limited() -> HelperError {
    HelperError::Remote("rate limited; retry in 42s".into())
}

fn accepting_one_call_then_rate_limited() -> (ScriptedLauncher, Pushed) {
    let pushed = Pushed::default();
    let seen = pushed.clone();
    let calls = Rc::new(Cell::new(0));
    let launcher = ScriptedLauncher::launching(move || {
        let calls = calls.clone();
        ScriptedHelper {
            push_answer: Box::new(move |ms| {
                calls.set(calls.get() + 1);
                if calls.get() > 1 {
                    return Err(rate_limited());
                }
                Ok(ms
                    .iter()
                    .map(|m| MutationOutcome {
                        oid: m.oid.clone(),
                        verdict: MutationVerdict::Accepted { remote_id: None },
                    })
                    .collect())
            }),
            pushed: seen.clone(),
            ..ScriptedHelper::answering(Ok(Default::default()))
        }
    });
    (launcher, pushed)
}

#[test]
fn a_rate_limit_after_the_move_sends_no_removal_and_keeps_the_move() {
    for (parent, child) in BOTH_OID_ORDERS {
        let s = held_upstream(&[
            task_in(parent, "P", "Home/"),
            task_in(child, "c", "Home/P/"),
        ]);
        s.filed(task_in(child, "c", "Home/")).deleted(&oid(parent));
        let (l, pushed) = accepting_one_call_then_rate_limited();
        let err = s.dam().push(remotes_launched_by(&l), None).unwrap_err();
        assert_eq!(err, UseCaseError::Helper(rate_limited()));
        let sent_before_the_stop: Vec<_> = pushed
            .borrow()
            .iter()
            .map(|m| (m.oid.clone(), m.op))
            .collect();
        assert_eq!(sent_before_the_stop, vec![(oid(child), Update)]);
        let upstream = s.store.remote_snapshot(&todoist(), &oid(child)).unwrap();
        assert_eq!(upstream.unwrap().base().path.as_str(), "Home/");
        assert!(
            s.store
                .remote_id(&todoist(), &oid(parent))
                .unwrap()
                .is_some()
        );
        assert_eq!(sent(&s).last(), Some(&(oid(parent), Delete)));
    }
}
