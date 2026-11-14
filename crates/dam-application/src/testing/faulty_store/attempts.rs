use dam_domain::Oid;

use super::FaultyStore;
use crate::ports::{Attempt, AttemptRepository, Attempts, RemoteName, StoreError};

impl AttemptRepository for FaultyStore {
    fn attempts(&self, remote: &RemoteName) -> Result<Attempts, StoreError> {
        self.inner.attempts(remote)
    }

    fn set_attempts(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        attempts: &[Attempt],
    ) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_attempts(remote, oid, attempts)
    }
}
