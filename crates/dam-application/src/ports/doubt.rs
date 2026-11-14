use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Field, Oid};

use super::{RemoteName, StoreError};

pub type FieldsInDoubt = BTreeMap<Oid, BTreeSet<Field>>;

pub trait DoubtRepository {
    fn fields_in_doubt(&self, remote: &RemoteName) -> Result<FieldsInDoubt, StoreError>;
    fn set_fields_in_doubt(
        &self,
        remote: &RemoteName,
        oid: &Oid,
        fields: &BTreeSet<Field>,
    ) -> Result<(), StoreError>;
}
