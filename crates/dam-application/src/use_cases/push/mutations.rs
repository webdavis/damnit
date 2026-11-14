use std::collections::BTreeSet;

use dam_domain::{Change, Field, changed_fields};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{FieldsInDoubt, Store};
use crate::remote::{MutationOp, RemoteCapabilities, RemoteMutation};
use crate::use_cases::push::idempotency::key;
use crate::use_cases::push::owed::OwedChanges;
use crate::use_cases::push::refiles::{Refiled, Refiles};
use crate::use_cases::push::sending_order::turn;

pub(super) struct Plan {
    pub(super) mutations: Vec<RemoteMutation>,
    pub(super) skipped: usize,
}

pub(super) fn plan(
    store: &dyn Store,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    mut changes: OwedChanges,
    refiles: &Refiles,
    doubt: &FieldsInDoubt,
) -> Result<Plan, UseCaseError> {
    let mut plan = Plan {
        mutations: Vec::new(),
        skipped: 0,
    };
    refiles.owe(&mut changes);
    let mut owed: Vec<Change> = changes.into_values().collect();
    owed.sort_by_key(turn);
    for change in owed {
        let refiled = refiles.refiled(store, &remote.name, change.oid())?;
        let in_doubt = doubt.get(change.oid()).cloned().unwrap_or_default();
        match mutation_for(store, caps, remote, &change, refiled, &in_doubt)? {
            Some(mut mutation) => {
                let answers = store.answers(&remote.name, &mutation.oid)?;
                mutation.idempotency_key = key(answers, &mutation);
                plan.mutations.push(mutation);
            }
            None => plan.skipped += 1,
        }
    }
    Ok(plan)
}

fn mutation_for(
    store: &dyn Store,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
    change: &Change,
    refiled: Option<Refiled>,
    in_doubt: &BTreeSet<Field>,
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
    let remote_id = store.remote_id(&remote.name, change.oid())?;
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
                        || in_doubt.contains(f)
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
