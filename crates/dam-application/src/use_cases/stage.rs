use std::collections::BTreeSet;

use dam_domain::{Change, Oid};

use crate::errors::UseCaseError;
use crate::ports::{CommitRepository, ObjectRepository};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::no_such_object;

impl Dam<'_> {
    pub fn add(&self, oids: &[Oid]) -> Result<Vec<Change>, UseCaseError> {
        atomically(self.store, || self.stage_each(oids))
    }

    pub fn add_all(&self) -> Result<Vec<Change>, UseCaseError> {
        atomically(self.store, || {
            let list: Vec<Oid> = working_and_ever_committed_oids(self.store, self.store)?
                .into_iter()
                .collect();
            self.stage_each(&list)
        })
    }

    pub fn reset(&self, oids: &[Oid]) -> Result<(), UseCaseError> {
        if oids.is_empty() {
            self.store.unstage_all()?;
        }
        for oid in oids {
            self.store.unstage(oid)?;
        }
        Ok(())
    }

    fn stage_each(&self, oids: &[Oid]) -> Result<Vec<Change>, UseCaseError> {
        let mut staged = Vec::new();
        for oid in oids {
            let working = self.store.get(oid)?;
            let committed = self.store.committed(oid)?;
            if working.is_none() && committed.is_none() {
                return Err(no_such_object(oid).into());
            }
            match Change::between(committed, working) {
                Some(change) => {
                    self.store.put_staged(change.clone())?;
                    staged.push(change);
                }
                None => self.store.unstage(oid)?,
            }
        }
        Ok(staged)
    }
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

#[cfg(test)]
mod tests;
