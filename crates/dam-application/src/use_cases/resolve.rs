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
    let (word, before, after) = match side {
        Side::Theirs => {
            store.put(&conflict.theirs)?;
            ("theirs", conflict.ours, conflict.theirs)
        }
        Side::Ours => ("ours", conflict.theirs, conflict.ours),
    };
    // Either side needs a commit, and an unpushed one. Push coalesces every
    // unpushed change for an object and sends where they end, so a resolution
    // that wrote none would leave the change that lost as the last word and
    // send that upstream instead.
    let record = CommitRecord {
        id: CommitId::generate(&mut |b| random.fill(b)),
        message: format!("resolve {} with {word}", oid.short()),
        at: clock.now(),
        changes: vec![Change {
            oid: oid.clone(),
            op: Op::Update,
            before: Some(before),
            after: Some(after),
        }],
    };
    store.commit(&record)?;
    store.clear_conflict(oid)?;
    Ok(())
}

#[cfg(test)]
mod tests;
