use dam_domain::Oid;

use super::MemoryStore;
use crate::ports::{Attempt, AttemptRepository, Attempts, RemoteName, StoreError};

impl AttemptRepository for MemoryStore {
    fn attempts(&self, remote: &RemoteName) -> Result<Attempts, StoreError> {
        Ok(self
            .0
            .borrow()
            .attempts
            .iter()
            .filter(|((held_for, _), _)| held_for == remote)
            .map(|((_, oid), attempts)| (oid.clone(), attempts.clone()))
            .collect())
    }

    fn set_attempts(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        attempts: &[Attempt],
    ) -> Result<(), StoreError> {
        let key = (remote.clone(), oid.clone());
        let record = &mut self.0.borrow_mut().attempts;
        match attempts.is_empty() {
            true => record.remove(&key),
            false => record.insert(key, attempts.to_vec()),
        };
        Ok(())
    }
}
