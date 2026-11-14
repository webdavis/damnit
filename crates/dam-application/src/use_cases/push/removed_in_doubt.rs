use dam_domain::{CommitRecord, Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::{FieldsInDoubt, RemoteName, Store};
use crate::use_cases::push::reconcile::Settlement;

pub(super) fn creates_in_doubt_removed(
    store: &dyn Store,
    remote: &RemoteName,
    unpushed: &[CommitRecord],
    doubt: &FieldsInDoubt,
) -> Result<Vec<Object>, UseCaseError> {
    let mut removed = Vec::new();
    for oid in doubt.keys() {
        if store.remote_id(remote, oid)?.is_some() || store.committed(oid)?.is_some() {
            continue;
        }
        removed.extend(last_known(unpushed, oid));
    }
    Ok(removed)
}

fn last_known(unpushed: &[CommitRecord], oid: &Oid) -> Option<Object> {
    unpushed
        .iter()
        .rev()
        .flat_map(|record| record.changes.iter().rev())
        .map(|c| &c.change)
        .filter(|change| change.oid() == oid)
        .find_map(|change| change.after().or(change.before()))
        .cloned()
}

impl Settlement {
    pub(super) fn forget_creates_in_doubt(&mut self, removed: Vec<Object>) {
        for object in removed {
            self.clear.push(object.base().oid.clone());
            self.removed_in_doubt.push(object);
        }
    }
}
