use std::collections::BTreeSet;

use dam_domain::Oid;

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ObjectRepository, StageRepository};

/// What restoring one oid did. `changed` is false when the working copy already
/// equalled the last commit and the stage held nothing for it.
#[derive(Debug, PartialEq, Eq)]
pub struct Restored {
    pub oid: Oid,
    pub changed: bool,
}

/// Sets each object's working copy back to its last commit and drops whatever
/// the stage held for it, so afterwards the object equals the commit and
/// nothing about it is staged.
///
/// Every oid is read before any is written: restoring throws work away, so an
/// object with no commit behind it refuses the whole command rather than
/// leaving half of it done. A repeated oid is one object and answers once.
pub fn restore(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    oids: &[Oid],
) -> Result<Vec<Restored>, UseCaseError> {
    let mut seen = BTreeSet::new();
    let oids: Vec<&Oid> = oids
        .iter()
        .filter(|oid| seen.insert((*oid).clone()))
        .collect();
    let mut committed = Vec::with_capacity(oids.len());
    for oid in &oids {
        match objects.committed(oid)? {
            Some(object) => committed.push(object),
            // An object still in the working layer was never committed, so
            // dam rm is the way to drop it. One in neither layer names
            // nothing: its delete has been committed, or the oid is wrong.
            None if objects.get(oid)?.is_some() => {
                return Err(Refusal::NotCommitted((*oid).clone()).into());
            }
            None => return Err(Refusal::NoSuchObject(oid.short().to_string()).into()),
        }
    }
    let staged: BTreeSet<Oid> = stage.staged()?.into_iter().map(|c| c.oid).collect();
    let mut restored = Vec::with_capacity(oids.len());
    for (oid, object) in oids.into_iter().zip(committed) {
        let changed = staged.contains(oid) || objects.get(oid)?.as_ref() != Some(&object);
        if changed {
            objects.put(&object)?;
            stage.unstage(oid)?;
        }
        restored.push(Restored {
            oid: oid.clone(),
            changed,
        });
    }
    Ok(restored)
}

#[cfg(test)]
mod tests;
