use dam_domain::{Change, CommitId, CommitRecord, Oid, Op};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, CommitRepository, ConflictRepository, ObjectRepository, Randomness};

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
    let conflict = conflicts
        .conflicts()?
        .into_iter()
        .find(|c| c.oid == *oid)
        .ok_or_else(|| Refusal::NoSuchObject(format!("conflict on {}", oid.short())))?;
    let (word, before, after) = match side {
        Side::Theirs => {
            objects.put(&conflict.theirs)?;
            ("theirs", conflict.ours, conflict.theirs)
        }
        Side::Ours => ("ours", conflict.theirs, conflict.ours),
    };
    let resolution = Change {
        oid: oid.clone(),
        op: Op::Update,
        before: Some(before),
        after: Some(after),
    };
    let message = format!("resolve {} with {word}", oid.short());
    commit_as_the_last_unpushed_word(commits, clock, random, message, resolution)?;
    conflicts.clear_conflict(oid)?;
    Ok(())
}

fn commit_as_the_last_unpushed_word(
    commits: &dyn CommitRepository,
    clock: &dyn Clock,
    random: &dyn Randomness,
    message: String,
    change: Change,
) -> Result<(), UseCaseError> {
    commits.commit(&CommitRecord {
        id: CommitId::generate(&mut |b| random.fill(b)),
        message,
        at: clock.now(),
        changes: vec![change],
    })?;
    Ok(())
}

#[cfg(test)]
mod tests;
