use std::collections::BTreeMap;

use dam_domain::{Oid, Op, Origin};

use super::MemoryStore;
use crate::ports::{OriginRepository, StoreError};

impl OriginRepository for MemoryStore {
    fn origins(&self) -> Result<BTreeMap<Oid, Origin>, StoreError> {
        let mut origins = BTreeMap::new();
        for change in self.0.borrow().commits.iter().flat_map(|c| &c.changes) {
            if change.change.op() == Op::Create {
                origins
                    .entry(change.change.oid().clone())
                    .or_insert_with(|| change.origin());
            }
        }
        Ok(origins)
    }
}
