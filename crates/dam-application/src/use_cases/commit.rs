use dam_domain::{Authored, CommitId, CommitRecord};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, Randomness};
use crate::service::Dam;

impl Dam<'_> {
    pub fn commit(&self, message: &str) -> Result<CommitRecord, UseCaseError> {
        let changes = self.store.staged()?;
        if changes.is_empty() {
            return Err(Refusal::NothingToCommit.into());
        }
        let record = record(self.clock, self.random, message.to_string(), changes);
        self.store.commit(&record)?;
        Ok(record)
    }
}

pub(crate) fn record(
    clock: &dyn Clock,
    random: &dyn Randomness,
    message: String,
    changes: Vec<Authored>,
) -> CommitRecord {
    CommitRecord {
        id: CommitId::generate(&mut |b| random.fill(b)),
        message,
        at: clock.now(),
        changes,
    }
}

#[cfg(test)]
mod tests;
