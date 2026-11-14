use dam_domain::Oid;

use super::FaultyStore;
use crate::ports::{AnswerRepository, RemoteName, StoreError};

impl AnswerRepository for FaultyStore {
    fn answers(&self, remote: &RemoteName, oid: &Oid) -> Result<u64, StoreError> {
        self.inner.answers(remote, oid)
    }

    fn count_answer(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.count_answer(remote, oid)
    }
}
