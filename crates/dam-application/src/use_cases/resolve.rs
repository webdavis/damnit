use dam_domain::{Change, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, CommitRepository, ConflictRepository, ObjectRepository, Randomness};
use crate::use_cases::atomically::atomically;
use crate::use_cases::commit::record;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Ours,
    Theirs,
}

pub fn resolve(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
    conflicts: &dyn ConflictRepository,
    clock: &dyn Clock,
    random: &dyn Randomness,
    oid: &Oid,
    side: Side,
) -> Result<(), UseCaseError> {
    atomically(objects, || {
        let conflict = conflicts
            .conflicts()?
            .into_iter()
            .find(|c| c.oid == *oid)
            .ok_or_else(|| Refusal::NoConflict(oid.clone()))?;
        let (word, before, after) = match side {
            Side::Theirs => {
                objects.put(&conflict.theirs)?;
                ("theirs", conflict.ours, conflict.theirs)
            }
            Side::Ours => ("ours", conflict.theirs, conflict.ours),
        };
        let resolution = Change::Update { before, after };
        let message = format!("resolve {} with {word}", oid.short());
        commits.commit(&record(clock, random, message, vec![resolution]))?;
        conflicts.clear_conflict(oid)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests;
