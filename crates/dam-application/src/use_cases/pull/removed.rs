use dam_domain::Oid;

use super::removal::{AtTheRemoval, Removal};
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Notice, Store};
use crate::remote::MutationOp;
use crate::use_cases::last_seen::LastSeen;

pub(super) fn record_removals(
    store: &dyn Store,
    remote: &RemoteConfig,
    removed: Vec<String>,
) -> Result<usize, UseCaseError> {
    let mut in_the_app = 0;
    if removed.is_empty() {
        return Ok(in_the_app);
    }
    let name = &remote.name;
    let mut record = store.attempts(name)?;
    let mut retries = store.push_retries(name)?;
    let retried_before = retries.len();
    for remote_id in removed {
        let Some(oid) = store.oid_for_remote_id(name, &remote_id)? else {
            continue;
        };
        let attempts = record.remove(&oid).unwrap_or_default();
        if !attempts.is_empty() {
            store.set_attempts(name, &oid, &[])?;
            store.count_answer(name, &oid)?;
        }
        if !retries.contains(&oid) {
            retries.push(oid.clone());
        }
        forget_failed_pushes(store, remote, &oid)?;
        let removal = Removal::of(AtTheRemoval {
            dam_sent_a_delete: attempts.iter().any(|a| a.op == MutationOp::Delete),
            committed: store.committed(&oid)?,
            in_the_working_copy: store.get(&oid)?.is_some(),
        });
        match &removal.kept_as_owing_nothing {
            Some(kept) => LastSeen::removed_in_the_app_nothing_owed(store, name, kept)?,
            None => LastSeen::gone(store, name, &oid)?,
        }
        if removal.noticed {
            notify(store, remote, &oid)?;
            in_the_app += 1;
        }
    }
    if retries.len() > retried_before {
        store.set_push_retries(name, &retries)?;
    }
    Ok(in_the_app)
}

fn notify(store: &dyn Store, remote: &RemoteConfig, oid: &Oid) -> Result<(), UseCaseError> {
    let subject = store
        .get(oid)?
        .map(|o| o.base().subject.clone())
        .unwrap_or_default();
    store.add_notice(&Notice::RemovedUpstream {
        remote: remote.name.clone(),
        oid: oid.clone(),
        subject,
    })?;
    Ok(())
}

fn forget_failed_pushes(
    store: &dyn Store,
    remote: &RemoteConfig,
    oid: &Oid,
) -> Result<(), UseCaseError> {
    for notice in store.notices()? {
        if matches!(&notice, Notice::PushFailed { remote: r, oid: o, .. } if *r == remote.name && o == oid)
        {
            store.remove_notice(&notice)?;
        }
    }
    Ok(())
}
