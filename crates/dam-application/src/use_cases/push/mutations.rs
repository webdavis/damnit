use std::collections::BTreeMap;

use dam_domain::{Change, CommitId, Field, Oid, changed_fields};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{CommitRepository, ObjectRepository, RemoteTrackingRepository};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};
use crate::use_cases::push::idempotency::{Origin, key};

pub(super) fn changes_to_send(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
    remote_tracking: &dyn RemoteTrackingRepository,
    remote: &RemoteConfig,
    unpushed: &[dam_domain::CommitRecord],
) -> Result<OwedChanges, UseCaseError> {
    let mut changes = net_changes_with_the_commit_of_their_last_change(unpushed);
    declare_each_retry_since_the_remote_snapshot(
        objects,
        commits,
        remote_tracking,
        remote,
        &mut changes,
    )?;
    Ok(changes)
}

type OwedChanges = BTreeMap<Oid, (Change, Option<CommitId>)>;

fn net_changes_with_the_commit_of_their_last_change(
    unpushed: &[dam_domain::CommitRecord],
) -> OwedChanges {
    let mut changes = OwedChanges::new();
    for record in unpushed {
        for change in &record.changes {
            let merged = match changes.remove(change.oid()) {
                Some((first, _)) => {
                    Change::between(first.before().cloned(), change.after().cloned())
                }
                None => Some(change.clone()),
            };
            if let Some(c) = merged {
                changes.insert(c.oid().clone(), (c, Some(record.id.clone())));
            }
        }
    }
    changes
}

fn declare_each_retry_since_the_remote_snapshot(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
    remote_tracking: &dyn RemoteTrackingRepository,
    remote: &RemoteConfig,
    changes: &mut OwedChanges,
) -> Result<(), UseCaseError> {
    for oid in commits.push_retries(&remote.name)? {
        let snapshot = remote_tracking.remote_snapshot(&remote.name, &oid)?;
        let owed = match changes.remove(&oid) {
            Some((owed, origin)) => Some((owed.after().cloned(), origin)),
            None => objects
                .committed(&oid)?
                .map(|current| (Some(current), None)),
        };
        if let Some((after, origin)) = owed
            && let Some(declared) = Change::between(snapshot, after)
        {
            changes.insert(oid, (declared, origin));
        }
    }
    Ok(())
}

pub(super) struct Plan {
    pub(super) mutations: Vec<RemoteMutation>,
    pub(super) skipped: usize,
}

pub(super) fn plan(
    remote_tracking: &dyn RemoteTrackingRepository,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    changes: OwedChanges,
) -> Result<Plan, UseCaseError> {
    let mut plan = Plan {
        mutations: Vec::new(),
        skipped: 0,
    };
    for (change, origin) in changes.into_values() {
        match mutation_for(remote_tracking, caps, remote, &change, origin.as_ref())? {
            Some(mutation) => plan.mutations.push(mutation),
            None => plan.skipped += 1,
        }
    }
    Ok(plan)
}

fn mutation_for(
    remote_tracking: &dyn RemoteTrackingRepository,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    change: &Change,
    origin: Option<&CommitId>,
) -> Result<Option<RemoteMutation>, UseCaseError> {
    let object = change.object();
    if !caps.kinds.contains(&object.kind()) {
        return Ok(None);
    }
    if let Some(scope) = &remote.path
        && !object.base().path.is_within(scope)
    {
        return Ok(None);
    }
    let remote_id = remote_tracking.remote_id(&remote.name, change.oid())?;
    let the_remote_holds_it = remote_id.is_some();
    let (op, fields): (MutationOp, Vec<Field>) = match change {
        Change::Delete { .. } => (MutationOp::Delete, vec![]),
        _ if !the_remote_holds_it => (MutationOp::Create, caps.fields.clone()),
        Change::Create { .. } => (MutationOp::Update, caps.fields.clone()),
        Change::Update { before, after } => {
            let declared: Vec<Field> = changed_fields(before, after)
                .into_iter()
                .filter(|f| caps.fields.contains(f))
                .collect();
            if declared.is_empty() {
                return Ok(None);
            }
            (MutationOp::Update, declared)
        }
    };
    if op == MutationOp::Delete && !the_remote_holds_it {
        return Ok(None);
    }
    Ok(Some(RemoteMutation {
        op,
        oid: change.oid().clone(),
        idempotency_key: key(origin.map_or(Origin::Retry, Origin::Commit), change.oid()),
        remote_id,
        object: if op == MutationOp::Delete {
            None
        } else {
            change.after().cloned()
        },
        fields,
    }))
}
