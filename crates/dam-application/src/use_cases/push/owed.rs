use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, CommitRecord, Oid};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Attempts, RemoteName, Store};
use crate::use_cases::last_seen::LastSeen;

pub(super) type OwedChanges = BTreeMap<Oid, Change>;

pub(super) fn changes_to_send(
    store: &dyn Store,
    remote: &RemoteConfig,
    unpushed: &[CommitRecord],
    record: &Attempts,
) -> Result<OwedChanges, UseCaseError> {
    let mut changes = net_changes(unpushed);
    let mut retried: BTreeSet<Oid> = store.push_retries(&remote.name)?.into_iter().collect();
    retried.extend(record.keys().cloned());
    retried.extend(removed_in_the_app_nothing_owed(
        store,
        &remote.name,
        &changes,
    )?);
    declare_since_the_remote_snapshot(store, remote, retried, &mut changes)?;
    owe_a_write_for_each_object_attempted(store, record, &mut changes)?;
    Ok(changes)
}

pub(super) fn follow_ups(
    store: &dyn Store,
    remote: &RemoteName,
    created: &[Oid],
) -> Result<OwedChanges, UseCaseError> {
    let mut still_attempted = store.attempts(remote)?;
    still_attempted.retain(|oid, _| created.contains(oid));
    let mut changes = OwedChanges::new();
    owe_a_write_for_each_object_attempted(store, &still_attempted, &mut changes)?;
    Ok(changes)
}

fn owe_a_write_for_each_object_attempted(
    store: &dyn Store,
    record: &Attempts,
    changes: &mut OwedChanges,
) -> Result<(), UseCaseError> {
    for oid in record.keys() {
        if changes.contains_key(oid) {
            continue;
        }
        if let Some(committed) = store.committed(oid)? {
            let unchanged = Change::Update {
                before: committed.clone(),
                after: committed,
            };
            changes.insert(oid.clone(), unchanged);
        }
    }
    Ok(())
}

fn net_changes(unpushed: &[CommitRecord]) -> OwedChanges {
    let mut changes = OwedChanges::new();
    for record in unpushed {
        for change in record.changes.iter().map(|c| &c.change) {
            let merged = match changes.remove(change.oid()) {
                Some(first) => Change::between(first.before().cloned(), change.after().cloned()),
                None => Some(change.clone()),
            };
            if let Some(c) = merged {
                changes.insert(c.oid().clone(), c);
            }
        }
    }
    changes
}

fn removed_in_the_app_nothing_owed(
    store: &dyn Store,
    remote: &RemoteName,
    changes: &OwedChanges,
) -> Result<Vec<Oid>, UseCaseError> {
    let mut removed = Vec::new();
    for oid in changes.keys() {
        if let LastSeen::RemovedInTheAppNothingOwed(_) = LastSeen::read(store, remote, oid)? {
            removed.push(oid.clone());
        }
    }
    Ok(removed)
}

fn declare_since_the_remote_snapshot(
    store: &dyn Store,
    remote: &RemoteConfig,
    retried: BTreeSet<Oid>,
    changes: &mut OwedChanges,
) -> Result<(), UseCaseError> {
    for oid in retried {
        let snapshot = LastSeen::baseline(store, &remote.name, &oid)?;
        let after = match changes.remove(&oid) {
            Some(owed) => owed.after().cloned(),
            None => store.committed(&oid)?,
        };
        if let Some(declared) = Change::between(snapshot, after) {
            changes.insert(oid, declared);
        }
    }
    Ok(())
}
