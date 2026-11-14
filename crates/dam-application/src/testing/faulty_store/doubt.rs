use std::collections::BTreeSet;

use dam_domain::{Field, Oid};

use super::FaultyStore;
use crate::ports::{DoubtRepository, FieldsInDoubt, RemoteName, StoreError};

impl DoubtRepository for FaultyStore {
    fn fields_in_doubt(&self, remote: &RemoteName) -> Result<FieldsInDoubt, StoreError> {
        self.inner.fields_in_doubt(remote)
    }

    fn set_fields_in_doubt(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        fields: &BTreeSet<Field>,
    ) -> Result<(), StoreError> {
        self.write()?;
        self.inner.set_fields_in_doubt(remote, oid, fields)
    }
}
