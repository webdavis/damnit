use dam_domain::{Field, Kind};

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
    fn accepted_fields(&self, remote: &RemoteName) -> Result<Option<Vec<Field>>, StoreError> {
        Ok(self.0.borrow().accepted_fields.get(remote).cloned())
    }
    fn set_accepted_fields(&self, remote: &RemoteName, fields: &[Field]) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .accepted_fields
            .insert(remote.clone(), fields.to_vec());
        Ok(())
    }
}
