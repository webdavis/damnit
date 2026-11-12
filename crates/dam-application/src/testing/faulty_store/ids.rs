use dam_domain::{CallerId, IdHolder, IdKey, Oid};

use super::FaultyStore;
use crate::ports::{IdRepository, StoreError};

impl IdRepository for FaultyStore {
    fn oid_for_id(&self, key: &IdKey) -> Result<Option<Oid>, StoreError> {
        self.inner.oid_for_id(key)
    }
    fn id_for_oid(&self, holder: &IdHolder, oid: &Oid) -> Result<Option<CallerId>, StoreError> {
        self.inner.id_for_oid(holder, oid)
    }
    fn map_id(&self, key: &IdKey, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.map_id(key, oid)
    }
}
