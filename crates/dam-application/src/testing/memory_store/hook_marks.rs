use dam_domain::Oid;

use super::MemoryStore;
use crate::config::HookEvent;
use crate::ports::{HookMarkRepository, StoreError};

impl HookMarkRepository for MemoryStore {
    fn mark_hooked(&self, oid: &Oid, event: HookEvent) -> Result<(), StoreError> {
        self.0.borrow_mut().hook_marks.insert(oid.clone(), event);
        Ok(())
    }

    fn hooked(&self, oid: &Oid) -> Result<Option<HookEvent>, StoreError> {
        Ok(self.0.borrow().hook_marks.get(oid).copied())
    }

    fn forget_hooked(&self, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().hook_marks.remove(oid);
        Ok(())
    }
}
