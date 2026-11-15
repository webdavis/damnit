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
    let mut owed = touched(unpushed);
    owed.extend(store.push_retries(&remote.name)?);
    owed.extend(record.keys().cloned());
    let mut changes = declare_since_the_remote_snapshot(store, &remote.name, owed)?;
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

fn touched(unpushed: &[CommitRecord]) -> BTreeSet<Oid> {
    unpushed
        .iter()
        .flat_map(|record| record.changes.iter().map(|c| c.change.oid().clone()))
        .collect()
}

fn declare_since_the_remote_snapshot(
    store: &dyn Store,
    remote: &RemoteName,
    owed: BTreeSet<Oid>,
) -> Result<OwedChanges, UseCaseError> {
    let mut changes = OwedChanges::new();
    for oid in owed {
        let snapshot = LastSeen::baseline(store, remote, &oid)?;
        if let Some(declared) = Change::between(snapshot, store.committed(&oid)?) {
            changes.insert(oid, declared);
        }
    }
    Ok(changes)
}
