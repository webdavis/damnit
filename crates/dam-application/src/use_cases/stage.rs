use std::collections::BTreeSet;

use dam_domain::{Change, Oid};

use crate::errors::UseCaseError;
use crate::ports::{CommitRepository, ObjectRepository, StageRepository};
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::no_such_object;

pub fn add(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    oids: &[Oid],
) -> Result<Vec<Change>, UseCaseError> {
    atomically(objects, || {
        for oid in oids {
            refuse_unknown(objects, oid)?;
        }
        stage_each(objects, stage, oids)
    })
}

fn refuse_unknown(objects: &dyn ObjectRepository, oid: &Oid) -> Result<(), UseCaseError> {
    if objects.get(oid)?.is_none() && objects.committed(oid)?.is_none() {
        return Err(no_such_object(oid).into());
    }
    Ok(())
}

fn stage_each(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    oids: &[Oid],
) -> Result<Vec<Change>, UseCaseError> {
    let mut staged = Vec::new();
    for oid in oids {
        match Change::between(objects.committed(oid)?, objects.get(oid)?) {
            Some(change) => {
                stage.put_staged(change.clone())?;
                staged.push(change);
            }
            None => stage.unstage(oid)?,
        }
    }
    Ok(staged)
}

pub(crate) fn working_and_ever_committed_oids(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
) -> Result<BTreeSet<Oid>, UseCaseError> {
    let mut oids: BTreeSet<Oid> = objects
        .all()?
        .into_iter()
        .map(|o| o.oid().clone())
        .collect();
    for record in commits.log()? {
        for change in record.changes {
            oids.insert(change.oid().clone());
        }
    }
    Ok(oids)
}

pub fn add_all(
    objects: &dyn ObjectRepository,
    stage: &dyn StageRepository,
    commits: &dyn CommitRepository,
) -> Result<Vec<Change>, UseCaseError> {
    atomically(objects, || {
        let list: Vec<Oid> = working_and_ever_committed_oids(objects, commits)?
            .into_iter()
            .collect();
        stage_each(objects, stage, &list)
    })
}

pub fn reset(stage: &dyn StageRepository, oids: &[Oid]) -> Result<(), UseCaseError> {
    if oids.is_empty() {
        stage.unstage_all()?;
    }
    for oid in oids {
        stage.unstage(oid)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
