use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Author, Authored, Change, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{ObjectRepository, Store};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::accept::Added;
use crate::use_cases::load::no_such_object;
use crate::use_cases::prefix::Named;

impl Dam<'_> {
    pub fn add(&self, named: &[Named]) -> Result<Added, UseCaseError> {
        atomically(self.store, || {
            let mut added = Added::default();
            let mut oids = Vec::new();
            for name in named {
                match name {
                    Named::Object(oid) => {
                        refuse_unknown(self.store, oid)?;
                        oids.push(oid.clone());
                    }
                    Named::Item(hash) => added.extend(self.accept(hash)?),
                }
            }
            added.staged.extend(self.stage_each(&oids)?);
            Ok(added)
        })
    }

    pub fn add_all(&self) -> Result<Added, UseCaseError> {
        atomically(self.store, || {
            let mut added = self.accept_every_item_that_applies()?;
            let list: Vec<Oid> = working_and_ever_committed_oids(self.store)?
                .into_iter()
                .collect();
            added.staged.extend(self.stage_each(&list)?);
            Ok(added)
        })
    }

    fn accept_every_item_that_applies(&self) -> Result<Added, UseCaseError> {
        let mut added = Added::default();
        for item in self.store.items()?.into_iter().filter(|i| i.is_waiting()) {
            match self.accept(&item.hash) {
                Ok(accepted) => added.extend(accepted),
                Err(UseCaseError::Refused(Refusal::Item(_))) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(added)
    }

    pub fn reset(&self, oids: &[Oid]) -> Result<(), UseCaseError> {
        atomically(self.store, || {
            let everything: Vec<Oid>;
            let oids = if oids.is_empty() {
                everything = self
                    .store
                    .staged()?
                    .into_iter()
                    .map(|s| s.change.oid().clone())
                    .collect();
                &everything
            } else {
                oids
            };
            for oid in oids {
                self.unpick(oid)?;
                self.store.unstage(oid)?;
            }
            Ok(())
        })
    }

    fn stage_each(&self, oids: &[Oid]) -> Result<Vec<Authored>, UseCaseError> {
        let sent: BTreeMap<Oid, Authored> = self
            .store
            .staged()?
            .into_iter()
            .filter(|s| s.author != Author::Owner)
            .map(|s| (s.change.oid().clone(), s))
            .collect();
        let mut staged = Vec::new();
        for oid in oids {
            let token_change = sent.get(oid);
            let base = match token_change {
                Some(sent) => sent.change.after().cloned(),
                None => self.store.committed(oid)?,
            };
            match Change::between(base, self.store.get(oid)?) {
                Some(change) => {
                    let authored = Authored::by_owner(change).with_details(self.access.details());
                    self.store.put_staged(authored.clone())?;
                    staged.push(authored);
                }
                None => {
                    self.store.unstage(oid)?;
                    if let Some(sent) = token_change {
                        self.store.put_staged(sent.clone())?;
                    }
                }
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
        for authored in record.changes {
            oids.insert(authored.change.oid().clone());
        }
    }
    Ok(oids)
}

#[cfg(test)]
mod tests;
