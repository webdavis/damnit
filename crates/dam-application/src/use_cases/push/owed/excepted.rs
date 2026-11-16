use dam_domain::{Change, CommitRecord, Object, Oid};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Store;
use crate::use_cases::push::owed::OwedChanges;

pub(in crate::use_cases::push) fn is_excepted(remote: &RemoteConfig, object: &Object) -> bool {
    let base = object.base();
    let owned = base.owned_folder();
    remote.except.iter().any(|except| {
        base.path.is_within(except)
            || owned
                .as_ref()
                .is_some_and(|folder| folder.is_within(except))
    })
}

pub(super) fn removals_kept_by_except(
    remote: &RemoteConfig,
    unpushed: &[CommitRecord],
    changes: &mut OwedChanges,
) {
    for (oid, change) in changes.iter_mut() {
        if !matches!(change, Change::Delete { .. }) {
            continue;
        }
        if let Some(before) = removed_here(unpushed, oid).filter(|o| is_excepted(remote, o)) {
            *change = Change::Delete { before };
        }
    }
}

fn removed_here(unpushed: &[CommitRecord], oid: &Oid) -> Option<Object> {
    unpushed
        .iter()
        .rev()
        .flat_map(|record| record.changes.iter().rev())
        .map(|c| &c.change)
        .find(|change| change.oid() == oid)
        .and_then(|change| match change {
            Change::Delete { before } => Some(before.clone()),
            _ => None,
        })
}

pub(in crate::use_cases::push) fn parent_left_out(
    store: &dyn Store,
    remote: &RemoteConfig,
    object: &Object,
) -> Result<Option<String>, UseCaseError> {
    let Some(parent) = &object.base().parent else {
        return Ok(None);
    };
    if store.remote_id(&remote.name, parent)?.is_some() {
        return Ok(None);
    }
    let left_out = store
        .committed(parent)?
        .is_some_and(|p| is_excepted(remote, &p));
    Ok(left_out.then(|| {
        let (child, parent) = (object.oid().short(), parent.short());
        format!(
            "{child}: its parent {parent} is under remote.{}.except, which this remote never \
             receives; run: dam mv {child} --parent PARENT, then dam push",
            remote.name.0
        )
    }))
}
