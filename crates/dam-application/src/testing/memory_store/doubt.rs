use std::collections::BTreeSet;

use dam_domain::{Field, Oid};

use super::MemoryStore;
use crate::ports::{DoubtRepository, FieldsInDoubt, RemoteName, StoreError};

impl DoubtRepository for MemoryStore {
    fn fields_in_doubt(&self, remote: &RemoteName) -> Result<FieldsInDoubt, StoreError> {
        Ok(self
            .0
            .borrow()
            .doubt
            .get(remote)
            .cloned()
            .unwrap_or_default())
    }

    fn set_fields_in_doubt(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        fields: &BTreeSet<Field>,
    ) -> Result<(), StoreError> {
        let mut inner = self.0.borrow_mut();
        let doubt = inner.doubt.entry(remote.clone()).or_default();
        match fields.is_empty() {
            true => doubt.remove(oid),
            false => doubt.insert(oid.clone(), fields.clone()),
        };
        Ok(())
    }
}
