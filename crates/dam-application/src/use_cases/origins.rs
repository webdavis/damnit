use std::collections::BTreeMap;

use dam_domain::{Oid, Op, Origin};

use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn origins(&self) -> Result<BTreeMap<Oid, Origin>, UseCaseError> {
        let mut origins = self.store.origins()?;
        for staged in self.store.staged()? {
            if staged.change.op() == Op::Create {
                origins
                    .entry(staged.change.oid().clone())
                    .or_insert_with(|| staged.origin());
            }
        }
        Ok(origins)
    }
}
