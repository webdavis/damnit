use std::collections::BTreeSet;

use dam_domain::{Change, Oid};

use crate::errors::UseCaseError;
use crate::ports::{ObjectRepository, Store};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::load::no_such_object;

impl Dam<'_> {
    pub fn add(&self, oids: &[Oid]) -> Result<Vec<Change>, UseCaseError> {
        atomically(self.store, || {
            for oid in oids {
                refuse_unknown(self.store, oid)?;
            }
            self.stage_each(oids)
        })
    }

    pub fn add_all(&self) -> Result<Vec<Change>, UseCaseError> {
        atomically(self.store, || {
            let list: Vec<Oid> = working_and_ever_committed_oids(self.store)?
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
            match Change::between(self.store.committed(oid)?, self.store.get(oid)?) {
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

fn refuse_unknown(objects: &dyn ObjectRepository, oid: &Oid) -> Result<(), UseCaseError> {
    if objects.get(oid)?.is_none() && objects.committed(oid)?.is_none() {
        return Err(no_such_object(oid).into());
    }
    Ok(())
}

pub(crate) fn working_and_ever_committed_oids(
    store: &dyn Store,
) -> Result<BTreeSet<Oid>, UseCaseError> {
    let mut oids: BTreeSet<Oid> = store.all()?.into_iter().map(|o| o.oid().clone()).collect();
    for record in store.log()? {
        for change in record.changes {
            oids.insert(change.oid().clone());
        }
    }
    Ok(oids)
}

#[cfg(test)]
mod tests;
