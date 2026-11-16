use std::collections::BTreeSet;

use dam_domain::{Change, Field, Object, Oid, changed_fields};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{Attempt, Attempts, RemoteHelper, RemoteTrackingRepository, Store};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};
use crate::use_cases::push::attempting::differing;
use crate::use_cases::push::containers::{Containers, Placement, places};
use crate::use_cases::push::owed::OwedChanges;
use crate::use_cases::push::sending_order::turn;

pub(super) struct Plan {
    pub(super) mutations: Vec<RemoteMutation>,
    pub(super) skipped: usize,
    pub(super) unused: Vec<Oid>,
    pub(super) unplaced: Vec<(Oid, String)>,
}

pub(super) fn plan(
    store: &dyn Store,
    helper: &dyn RemoteHelper,
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
        skipped: 0,
        unused: Vec::new(),
        unplaced: Vec::new(),
    };
    let mut owed: Vec<Change> = changes.into_values().collect();
    owed.sort_by_key(turn);
    for change in owed {
        let earlier = record.get(change.oid()).map_or(&[][..], Vec::as_slice);
        let differing = differing(helper, caps, change.after(), earlier)?;
        match mutation_for(remote_tracking, caps, remote, &change, &differing)? {
            Some(mut mutation) => {
                if let (true, Some(object)) = (places(&mutation), &mutation.object) {
                    match containers.place(remote_tracking, &remote.name, object, &creating)? {
                        Placement::Named(container) => mutation.container_remote_id = container,
                        Placement::CreatedThisPush(parent) => mutation.container_oid = Some(parent),
                        Placement::Held(why) => {
                            plan.unplaced.push((mutation.oid, why));
                            continue;
                        }
                    }
                }
                if mutation.op == MutationOp::Create {
                    creating.insert(mutation.oid.clone());
                }
                plan.mutations.push(mutation);
            }
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
                .filter(|f| changed.contains(f) || differing.contains(f))
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
        container_remote_id: None,
        container_oid: None,
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
