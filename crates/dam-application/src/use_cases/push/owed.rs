use std::collections::BTreeMap;

use dam_domain::{Change, CommitRecord, Oid};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{FieldsInDoubt, RemoteName, Store};

pub(super) type OwedChanges = BTreeMap<Oid, Change>;

pub(super) fn changes_to_send(
    store: &dyn Store,
    remote: &RemoteConfig,
    unpushed: &[CommitRecord],
) -> Result<OwedChanges, UseCaseError> {
    let mut changes = net_changes(unpushed);
    declare_each_retry_since_the_remote_snapshot(store, remote, &mut changes)?;
    let doubt = store.fields_in_doubt(&remote.name)?;
    owe_a_write_for_each_object_in_doubt(store, &doubt, &mut changes)?;
    Ok(changes)
}

pub(super) fn follow_ups(
    store: &dyn Store,
    remote: &RemoteName,
    created: &[Oid],
) -> Result<OwedChanges, UseCaseError> {
    let mut created_in_doubt = store.fields_in_doubt(remote)?;
    created_in_doubt.retain(|oid, _| created.contains(oid));
    let mut changes = OwedChanges::new();
    owe_a_write_for_each_object_in_doubt(store, &created_in_doubt, &mut changes)?;
    Ok(changes)
}

fn owe_a_write_for_each_object_in_doubt(
    store: &dyn Store,
    doubt: &FieldsInDoubt,
    changes: &mut OwedChanges,
) -> Result<(), UseCaseError> {
    for oid in doubt.keys() {
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

fn declare_each_retry_since_the_remote_snapshot(
    store: &dyn Store,
    remote: &RemoteConfig,
    changes: &mut OwedChanges,
) -> Result<(), UseCaseError> {
    for oid in store.push_retries(&remote.name)? {
        let snapshot = store.remote_snapshot(&remote.name, &oid)?;
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
