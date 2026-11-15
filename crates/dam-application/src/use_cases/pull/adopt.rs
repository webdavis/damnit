use std::collections::BTreeMap;

use dam_domain::Oid;

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Attempts, Declared, RemoteHelper, Store};
use crate::remote::{IncomingObject, MutationOp, RemoteCapabilities};
use crate::use_cases::as_it_stands::as_it_stands;

pub(super) type Adopted = BTreeMap<String, Oid>;

pub(super) fn creates_found(
    store: &dyn Store,
    remote: &RemoteConfig,
    helper: &dyn RemoteHelper,
    caps: &RemoteCapabilities,
    record: &Attempts,
    objects: &[IncomingObject],
) -> Result<Adopted, UseCaseError> {
    let in_doubt = creates_in_doubt(store, remote, record)?;
    if in_doubt.is_empty() {
        return Ok(Adopted::new());
    }
    let mut matches = Vec::new();
    for incoming in objects {
        let mapped = store.oid_for_remote_id(&remote.name, &incoming.remote_id)?;
        if mapped.is_some() {
            continue;
        }
        let theirs = as_it_stands(helper, caps, &incoming.object)?;
        for (oid, sent) in &in_doubt {
            if sent.contains(&&theirs) {
                matches.push((incoming.remote_id.clone(), (*oid).clone()));
            }
        }
    }
    Ok(one_to_one(matches))
}

fn creates_in_doubt<'a>(
    store: &dyn Store,
    remote: &RemoteConfig,
    record: &'a Attempts,
) -> Result<Vec<(&'a Oid, Vec<&'a Declared>)>, UseCaseError> {
    let mut in_doubt = Vec::new();
    for (oid, attempts) in record {
        let sent: Vec<&Declared> = attempts
            .iter()
            .filter(|a| a.op == MutationOp::Create)
            .map(|a| &a.declared)
            .collect();
        let unmapped = store.remote_id(&remote.name, oid)?.is_none();
        let still_here = store.committed(oid)?.is_some() && store.get(oid)?.is_some();
        if !sent.is_empty() && unmapped && still_here {
            in_doubt.push((oid, sent));
        }
    }
    Ok(in_doubt)
}

fn one_to_one(matches: Vec<(String, Oid)>) -> Adopted {
    let alone = |pair: &(String, Oid)| {
        matches.iter().filter(|m| m.0 == pair.0).count() == 1
            && matches.iter().filter(|m| m.1 == pair.1).count() == 1
    };
    matches.iter().filter(|m| alone(m)).cloned().collect()
}
