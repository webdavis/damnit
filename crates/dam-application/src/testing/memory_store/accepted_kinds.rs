use dam_domain::Kind;

use crate::ports::{AcceptedKindsRepository, RemoteName, StoreError};

use super::MemoryStore;

impl AcceptedKindsRepository for MemoryStore {
    fn accepted_kinds(&self, remote: &RemoteName) -> Result<Option<Vec<Kind>>, StoreError> {
        Ok(self.0.borrow().accepted_kinds.get(remote).cloned())
    }
    fn set_accepted_kinds(&self, remote: &RemoteName, kinds: &[Kind]) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .accepted_kinds
            .insert(remote.clone(), kinds.to_vec());
        Ok(())
    }
}
