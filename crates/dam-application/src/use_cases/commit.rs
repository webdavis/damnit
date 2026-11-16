use dam_domain::{Authored, CommitId, CommitRecord};

use crate::errors::{Refusal, UseCaseError};
use crate::ports::{Clock, Randomness};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;

impl Dam<'_> {
    pub fn commit(&self, message: &str) -> Result<CommitRecord, UseCaseError> {
        let changes = self.store.staged()?;
        if changes.is_empty() {
            return Err(Refusal::NothingToCommit.into());
        }
        let record = record(self.clock, self.random, message.to_string(), changes);
        atomically(self.store, || {
            self.store.commit(&record)?;
            for authored in &record.changes {
                self.store.forget_hooked(authored.change.oid())?;
            }
            Ok(())
        })?;
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
        pulled: false,
        at: clock.now(),
        changes,
    }
}

#[cfg(test)]
mod tests;
