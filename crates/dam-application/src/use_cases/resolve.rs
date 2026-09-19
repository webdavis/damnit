use dam_domain::{Change, CommitId, CommitRecord, Oid, Op};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, ObjectStore, Randomness};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Ours,
    Theirs,
}

pub fn resolve(
    store: &dyn ObjectStore,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
    oid: &Oid,
    side: Side,
) -> Result<(), UseCaseError> {
    let conflict = store
        .conflicts()?
        .into_iter()
        .find(|c| c.oid == *oid)
        .ok_or_else(|| Refusal::NoSuchObject(format!("conflict on {}", oid.short())))?;
    if side == Side::Theirs {
        store.put(&conflict.theirs)?;
        let record = CommitRecord {
            id: CommitId::generate(&mut |b| random.fill(b)),
            message: format!("resolve {} with theirs", oid.short()),
            at: clock.now(),
            changes: vec![Change {
                oid: oid.clone(),
                op: Op::Update,
                before: Some(conflict.ours.clone()),
                after: Some(conflict.theirs.clone()),
            }],
        };
        store.commit(&record)?;
        store.mark_pushed(&conflict.remote, &record.id)?;
    }
    store.clear_conflict(oid)?;
    Ok(())
}

#[cfg(test)]
mod tests;
