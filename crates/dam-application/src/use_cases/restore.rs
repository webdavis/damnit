use std::collections::BTreeSet;

use dam_domain::{Object, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ObjectRepository, StageRepository};
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::no_such_object;

#[derive(Debug, PartialEq, Eq)]
pub struct Restored {
    pub oid: Oid,
    pub changed: bool,
}

pub fn restore(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    oids: &[Oid],
) -> Result<Vec<Restored>, UseCaseError> {
    atomically(objects, || {
        let oids = each_once(oids);
        let committed = every_commit_before_writing_any(objects, &oids)?;
        let staged: BTreeSet<Oid> = stage
            .staged()?
            .into_iter()
            .map(|c| c.oid().clone())
            .collect();
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
    })
}

fn each_once(oids: &[Oid]) -> Vec<&Oid> {
    let mut seen = BTreeSet::new();
    oids.iter()
        .filter(|oid| seen.insert((*oid).clone()))
        .collect()
}

fn every_commit_before_writing_any(
    objects: &dyn ObjectRepository,
    oids: &[&Oid],
) -> Result<Vec<Object>, UseCaseError> {
    let mut committed = Vec::with_capacity(oids.len());
    for oid in oids {
        match objects.committed(oid)? {
            Some(object) => committed.push(object),
            None => return Err(never_committed_or_already_gone(objects, oid)?.into()),
        }
    }
    Ok(committed)
}

fn never_committed_or_already_gone(
    objects: &dyn ObjectRepository,
    oid: &Oid,
) -> Result<Refusal, UseCaseError> {
    let still_in_working_so_dam_rm_drops_it = objects.get(oid)?.is_some();
    Ok(if still_in_working_so_dam_rm_drops_it {
        Refusal::NotCommitted(oid.clone())
    } else {
        no_such_object(oid)
    })
}

#[cfg(test)]
mod tests;
