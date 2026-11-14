use std::collections::BTreeMap;

use dam_domain::Oid;

use crate::errors::UseCaseError;
use crate::ports::{Attempt, HelperError, RemoteHelper, RemoteName, Store};
use crate::remote::RemoteMutation;
use crate::use_cases::atomically::atomically;
use crate::use_cases::push::reconcile::Heard;

pub(super) type Attempting = BTreeMap<Oid, Attempt>;

pub(super) fn attempting(
    helper: &dyn RemoteHelper,
    mutations: &[RemoteMutation],
) -> Result<Attempting, HelperError> {
    mutations
        .iter()
        .map(|m| {
            let attempt = Attempt {
                key: m.idempotency_key.clone(),
                op: m.op,
                container_remote_id: m.container_remote_id.clone(),
                declared: helper.declared_on_the_wire(m)?,
            };
            Ok((m.oid.clone(), attempt))
        })
        .collect()
}

pub(super) fn record_before_the_call(
    store: &dyn Store,
    remote: &RemoteName,
    ready: &[RemoteMutation],
    attempting: &Attempting,
) -> Result<(), UseCaseError> {
    atomically(store, || {
        let mut record = store.attempts(remote)?;
        for (oid, attempt) in ready
            .iter()
            .filter_map(|m| attempting.get_key_value(&m.oid))
        {
            let attempts = record.entry(oid.clone()).or_default();
            attempts.push(attempt.clone());
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
            Heard::Accepted => attempts.clear(),
            Heard::Refused => attempts.retain(|a| a.key != sent.key),
        }
        store.set_attempts(remote, oid, attempts)?;
    }
    Ok(())
}
