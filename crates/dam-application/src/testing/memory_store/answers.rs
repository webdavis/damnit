use dam_domain::Oid;

use super::MemoryStore;
use crate::ports::{AnswerRepository, RemoteName, StoreError};

impl AnswerRepository for MemoryStore {
    fn answers(&self, remote: &RemoteName, oid: &Oid) -> Result<u64, StoreError> {
        let key = (remote.clone(), oid.clone());
        Ok(self.0.borrow().answers.get(&key).copied().unwrap_or(0))
    }

    fn count_answer(&self, remote: &RemoteName, oid: &Oid) -> Result<(), StoreError> {
        let key = (remote.clone(), oid.clone());
        *self.0.borrow_mut().answers.entry(key).or_insert(0) += 1;
        Ok(())
    }
}
