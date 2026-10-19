use dam_domain::{Change, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::commit::record;

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
                    self.store.put(&conflict.theirs)?;
                    ("theirs", conflict.ours, conflict.theirs)
                }
                Side::Ours => ("ours", conflict.theirs, conflict.ours),
            };
            let resolution = Change::Update { before, after };
            let message = format!("resolve {} with {word}", oid.short());
            self.store
                .commit(&record(self.clock, self.random, message, vec![resolution]))?;
            self.store.clear_conflict(oid)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests;
