use std::collections::BTreeMap;

use dam_domain::{Oid, Origin};

use super::FaultyStore;
use crate::ports::{OriginRepository, StoreError};

impl OriginRepository for FaultyStore {
    fn origins(&self) -> Result<BTreeMap<Oid, Origin>, StoreError> {
        self.inner.origins()
    }
}
