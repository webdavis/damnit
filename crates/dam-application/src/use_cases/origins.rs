use std::collections::BTreeMap;

use dam_domain::{Oid, Op, Origin};

use crate::errors::UseCaseError;
use crate::ports::{Store, StoreError};
use crate::service::Dam;

impl Dam<'_> {
    pub fn origins(&self) -> Result<BTreeMap<Oid, Origin>, UseCaseError> {
        Ok(origins_in(self.store)?)
    }
}

pub(crate) fn origins_in(store: &dyn Store) -> Result<BTreeMap<Oid, Origin>, StoreError> {
    let mut origins = store.origins()?;
    for staged in store.staged()? {
        if staged.change.op() == Op::Create {
            origins
                .entry(staged.change.oid().clone())
                .or_insert_with(|| staged.origin());
        }
    }
    Ok(origins)
}
