use dam_domain::Oid;

use super::StoreError;
use crate::config::HookEvent;

pub trait HookMarkRepository {
    fn mark_hooked(&self, oid: &Oid, event: HookEvent) -> Result<(), StoreError>;
    fn hooked(&self, oid: &Oid) -> Result<Option<HookEvent>, StoreError>;
    fn forget_hooked(&self, oid: &Oid) -> Result<(), StoreError>;
}
