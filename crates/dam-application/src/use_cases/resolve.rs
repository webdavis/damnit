use dam_domain::{Authored, Change, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::commit::record;
use crate::use_cases::parents::placed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Ours,
    Theirs,
}

impl Dam<'_> {
    pub fn resolve(&self, oid: &Oid, side: Side) -> Result<(), UseCaseError> {
        atomically(self.store, || {
            let conflict = self
                .store
                .conflicts()?
                .into_iter()
                .find(|c| c.oid == *oid)
                .ok_or_else(|| Refusal::NoConflict(oid.clone()))?;
            let (word, before, after) = match side {
                Side::Theirs => {
                    let theirs = placed(self.store, conflict.ours.as_ref(), conflict.theirs)?;
                    self.store.put(&theirs)?;
                    ("theirs", conflict.ours, Some(theirs))
                }
                Side::Ours => ("ours", Some(conflict.theirs), conflict.ours),
            };
            if let Some(change) = Change::between(before, after) {
                let resolution = Authored::by_owner(change).with_details(self.access.details());
                let message = format!("resolve {} with {word}", oid.short());
                self.store
                    .commit(&record(self.clock, self.random, message, vec![resolution]))?;
            }
            self.store.clear_conflict(oid)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests;
