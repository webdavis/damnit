use std::collections::BTreeSet;

use dam_domain::{Change, Field, Object, Oid, changed_fields};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Attempt, Attempts, RemoteTrackingRepository, Store};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};
use crate::use_cases::as_it_stands::Wire;
use crate::use_cases::push::attempting::differing;
use crate::use_cases::push::containers::Containers;
use crate::use_cases::push::hold::Hold;
use crate::use_cases::push::owed::OwedChanges;
use crate::use_cases::push::sending_order::turn;
use crate::use_cases::push::skip::{Skip, not_going};

pub(super) struct Plan {
    pub(super) mutations: Vec<RemoteMutation>,
    pub(super) skipped: Vec<(Oid, Skip)>,
    pub(super) unused: Vec<Oid>,
    pub(super) unplaced: Vec<Hold>,
}

pub(super) fn plan(
    store: &dyn Store,
    wire: Wire<'_>,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    changes: OwedChanges,
    record: &Attempts,
) -> Result<Plan, UseCaseError> {
    let remote_tracking: &dyn RemoteTrackingRepository = store;
    let containers = Containers::new(store, &changes)?;
    let mut creating = BTreeSet::new();
    let mut plan = Plan {
        mutations: Vec::new(),
        skipped: Vec::new(),
        unused: Vec::new(),
        unplaced: Vec::new(),
    };
    let mut owed: Vec<Change> = changes.into_values().collect();
    owed.sort_by_key(turn);
    for change in owed {
        let earlier = record.get(change.oid()).map_or(&[][..], Vec::as_slice);
        let differing = differing(wire, caps, change.after(), earlier)?;
        match mutation_for(remote_tracking, caps, remote, &change, &differing)? {
            Ok(mut mutation) => {
                if let Some(hold) = containers.file(store, remote, &mut mutation, &creating)? {
                    plan.unplaced.push(hold);
                    continue;
                }
                if mutation.op == MutationOp::Create {
                    creating.insert(mutation.oid.clone());
                }
                plan.mutations.push(mutation);
            }
            Err(_) if nothing_will_use(remote_tracking, caps, remote, &change, earlier)? => {
                plan.unused.push(change.oid().clone());
            }
            Err(skip) => plan.skipped.push((change.oid().clone(), skip)),
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
    differing: &BTreeSet<Field>,
) -> Result<Result<RemoteMutation, Skip>, UseCaseError> {
    if let Some(skip) = not_going(remote, caps, change.object()) {
        return Ok(Err(skip));
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
                .filter(|f| changed.contains(f) || differing.contains(f))
                .filter(|f| caps.fields.contains(f))
                .collect();
            if declared.is_empty() {
                return Ok(Err(Skip::NoFieldItTakes));
            }
            (MutationOp::Update, declared)
        }
    };
    if op == MutationOp::Delete && !the_remote_holds_it {
        return Ok(Err(Skip::NeverSent));
    }
    Ok(Ok(RemoteMutation {
        op,
        oid: change.oid().clone(),
        idempotency_key: String::new(),
        remote_id,
        container_remote_id: None,
        container_oid: None,
        object: change.after().cloned(),
        fields,
    }))
}

pub(super) fn goes_to(remote: &RemoteConfig, caps: &RemoteCapabilities, object: &Object) -> bool {
    not_going(remote, caps, object).is_none()
}
