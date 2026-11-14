use dam_domain::Oid;

use crate::errors::UseCaseError;
use crate::ports::{Attempt, RemoteName, Store};
use crate::remote::{MutationOp, RemoteMutation};
use crate::use_cases::atomically::atomically;
use crate::use_cases::push::attempting::Attempting;
use crate::use_cases::push::reconcile::Heard;

pub(super) fn record_before_the_call(
    store: &dyn Store,
    remote: &RemoteName,
    ready: &[RemoteMutation],
    attempting: &Attempting,
) -> Result<(), UseCaseError> {
    atomically(store, || {
        let mut record = store.attempts(remote)?;
        let new = ready
            .iter()
            .filter_map(|m| attempting.get_key_value(&m.oid))
            .filter(|(_, sending)| !sending.repeats_the_last);
        for (oid, sending) in new {
            let attempts = record.entry(oid.clone()).or_default();
            attempts.push(sending.attempt.clone());
            store.set_attempts(remote, oid, attempts)?;
        }
        Ok(())
    })
}

pub(super) fn settle_attempts(
    store: &dyn Store,
    remote: &RemoteName,
    heard: &[(Oid, Heard)],
    attempting: &Attempting,
) -> Result<(), UseCaseError> {
    let mut record = store.attempts(remote)?;
    for (oid, heard) in heard {
        let (Some(attempts), Some(sent)) = (record.get_mut(oid), attempting.get(oid)) else {
            continue;
        };
        match heard {
            Heard::Accepted if owes_a_follow_up(attempts, &sent.attempt) => {}
            Heard::Accepted => attempts.clear(),
            Heard::Refused => attempts.retain(|a| a.key != sent.attempt.key),
        }
        store.set_attempts(remote, oid, attempts)?;
    }
    Ok(())
}

fn owes_a_follow_up(attempts: &[Attempt], accepted: &Attempt) -> bool {
    let differs_from_what_was_accepted = |attempt: &Attempt| {
        attempt
            .declared
            .iter()
            .any(|(field, value)| accepted.declared.get(field).is_some_and(|v| v != value))
    };
    accepted.op == MutationOp::Create && attempts.iter().any(differs_from_what_was_accepted)
}
