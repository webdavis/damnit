use dam_domain::Oid;

use super::FaultyStore;
use crate::config::HookEvent;
use crate::ports::{HookMarkRepository, StoreError};

impl HookMarkRepository for FaultyStore {
    fn mark_hooked(&self, oid: &Oid, event: HookEvent) -> Result<(), StoreError> {
        self.write()?;
        self.inner.mark_hooked(oid, event)
    }

    fn hooked(&self, oid: &Oid) -> Result<Option<HookEvent>, StoreError> {
        self.inner.hooked(oid)
    }

    fn forget_hooked(&self, oid: &Oid) -> Result<(), StoreError> {
        self.write()?;
        self.inner.forget_hooked(oid)
    }
}
