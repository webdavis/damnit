use std::collections::BTreeSet;

use dam_domain::{Change, Field, Object, Oid, changed_fields};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Attempt, Attempts, RemoteHelper, RemoteTrackingRepository};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};
use crate::use_cases::push::attempting::differing;
use crate::use_cases::push::owed::OwedChanges;
use crate::use_cases::push::refiles::{Refiled, Refiles};
use crate::use_cases::push::sending_order::turn;

pub(super) struct Plan {
    pub(super) mutations: Vec<RemoteMutation>,
    pub(super) skipped: usize,
    pub(super) unused: Vec<Oid>,
}

pub(super) fn plan(
    remote_tracking: &dyn RemoteTrackingRepository,
    helper: &dyn RemoteHelper,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    mut changes: OwedChanges,
    refiles: &Refiles,
    record: &Attempts,
) -> Result<Plan, UseCaseError> {
    let mut plan = Plan {
        mutations: Vec::new(),
        skipped: 0,
        unused: Vec::new(),
    };
    refiles.owe(&mut changes);
    let mut owed: Vec<Change> = changes.into_values().collect();
    owed.sort_by_key(turn);
    for change in owed {
        let refiled = refiles.refiled(remote_tracking, &remote.name, change.oid())?;
        let earlier = record.get(change.oid()).map_or(&[][..], Vec::as_slice);
        let differing = differing(helper, caps, change.after(), earlier)?;
        match mutation_for(remote_tracking, caps, remote, &change, refiled, &differing)? {
            Some(mutation) => plan.mutations.push(mutation),
            None if nothing_will_use(remote_tracking, caps, remote, &change, earlier)? => {
                plan.unused.push(change.oid().clone());
            }
            None => plan.skipped += 1,
        }
    }
    Ok(plan)
}

fn nothing_will_use(
    remote_tracking: &dyn RemoteTrackingRepository,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    unsent: &Change,
    earlier: &[Attempt],
) -> Result<bool, UseCaseError> {
    if earlier.is_empty()
        || remote_tracking
            .remote_id(&remote.name, unsent.oid())?
            .is_none()
    {
        return Ok(false);
    }
    let holds_a_delete = earlier.iter().any(|a| a.op == MutationOp::Delete);
    Ok(unsent
        .after()
        .is_none_or(|now| goes_to(remote, caps, now) && !holds_a_delete))
}

fn mutation_for(
    remote_tracking: &dyn RemoteTrackingRepository,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    change: &Change,
    refiled: Option<Refiled>,
    differing: &BTreeSet<Field>,
) -> Result<Option<RemoteMutation>, UseCaseError> {
    if !goes_to(remote, caps, change.object()) {
        return Ok(None);
    }
    let remote_id = remote_tracking.remote_id(&remote.name, change.oid())?;
    let the_remote_holds_it = remote_id.is_some();
    let (op, fields): (MutationOp, Vec<Field>) = match change {
        Change::Delete { .. } => (MutationOp::Delete, vec![]),
        _ if !the_remote_holds_it => (MutationOp::Create, caps.fields.clone()),
        Change::Create { .. } => (MutationOp::Update, caps.fields.clone()),
        Change::Update { before, after } => {
            let changed = changed_fields(before, after);
            let declared: Vec<Field> = Field::ALL
                .into_iter()
                .filter(|f| {
                    changed.contains(f)
                        || differing.contains(f)
                        || (refiled.is_some() && *f == Field::Path)
                })
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
        idempotency_key: String::new(),
        remote_id,
        container_remote_id: refiled.and_then(|r| r.container_remote_id),
        object: change.after().cloned(),
        fields,
    }))
}

pub(super) fn goes_to(remote: &RemoteConfig, caps: &RemoteCapabilities, object: &Object) -> bool {
    let within_its_path = remote
        .path
        .as_ref()
        .is_none_or(|scope| object.base().path.is_within(scope));
    caps.kinds.contains(&object.kind()) && within_its_path
}
