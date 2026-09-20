//! What a push sends: which committed changes are owed to a remote, and the
//! one mutation each of them becomes.

use std::collections::BTreeMap;

use dam_domain::{Change, CommitId, Field, Oid, Op, changed_fields, coalesce};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{CommitRepository, ObjectRepository, RemoteTrackingRepository};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};
use crate::use_cases::push::idempotency::{Origin, key};

/// Every unpushed commit's changes, coalesced per oid and paired with the
/// commit the last of them came from, plus a synthetic update from the
/// committed view for each oid owed a retry that no unpushed commit already
/// covers.
///
/// A coalesced commit change already ends at the committed state, so leaving
/// it in place resends exactly what the synthetic one would have, and keeps
/// the commit that names it. That name is what lets a resend after an
/// interrupted push carry the key the interrupted one carried.
pub(super) fn changes_to_send(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
    remote: &RemoteConfig,
    unpushed: &[dam_domain::CommitRecord],
) -> Result<BTreeMap<Oid, (Change, Option<CommitId>)>, UseCaseError> {
    let mut changes: BTreeMap<Oid, (Change, Option<CommitId>)> = BTreeMap::new();
    for record in unpushed {
        for change in &record.changes {
            let merged = match changes.remove(&change.oid) {
                Some((previous, _)) => coalesce(previous, change.clone()),
                None => Some(change.clone()),
            };
            if let Some(c) = merged {
                changes.insert(c.oid.clone(), (c, Some(record.id.clone())));
            }
        }
    }
    for oid in commits.push_retries(&remote.name)? {
        if changes.contains_key(&oid) {
            continue;
        }
        if let Some(current) = objects.committed(&oid)? {
            changes.insert(
                oid.clone(),
                (
                    Change {
                        oid,
                        op: Op::Update,
                        before: None,
                        after: Some(current),
                    },
                    None,
                ),
            );
        }
    }
    Ok(changes)
}

pub(super) fn mutation_for(
    remote_tracking: &dyn RemoteTrackingRepository,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    change: &Change,
    origin: Option<&CommitId>,
) -> Result<Option<RemoteMutation>, UseCaseError> {
    let Some(object) = change.after.as_ref().or(change.before.as_ref()) else {
        return Ok(None);
    };
    if !caps.kinds.contains(&object.kind()) {
        return Ok(None);
    }
    if let Some(scope) = &remote.path
        && !object.base().path.is_within(scope)
    {
        return Ok(None);
    }
    let remote_id = remote_tracking.remote_id(&remote.name, &change.oid)?;
    // Whether the remote already holds the object decides the verb, not the
    // change's own op: a create the remote has already taken is an update, and
    // an update of something the remote has never seen is a create.
    let (op, fields): (MutationOp, Vec<Field>) = match (change.op, &change.before, &change.after) {
        (Op::Delete, _, _) => (MutationOp::Delete, vec![]),
        (Op::Update, _, None) => return Ok(None),
        (_, _, _) if remote_id.is_none() => (MutationOp::Create, caps.fields.clone()),
        (Op::Create, _, _) => (MutationOp::Update, caps.fields.clone()),
        (Op::Update, Some(before), Some(after)) => {
            let declared: Vec<Field> = changed_fields(before, after)
                .into_iter()
                .filter(|f| caps.fields.contains(f))
                .collect();
            if declared.is_empty() {
                return Ok(None);
            }
            (MutationOp::Update, declared)
        }
        (Op::Update, None, Some(_)) => (MutationOp::Update, caps.fields.clone()),
    };
    if op == MutationOp::Delete && remote_id.is_none() {
        return Ok(None);
    }
    Ok(Some(RemoteMutation {
        op,
        oid: change.oid.clone(),
        idempotency_key: key(origin.map_or(Origin::Retry, Origin::Commit), &change.oid),
        remote_id,
        object: if op == MutationOp::Delete {
            None
        } else {
            change.after.clone()
        },
        fields,
    }))
}
