use dam_domain::{CallerId, IdHolder, IdKey, Oid};

use super::MemoryStore;
use crate::ports::{IdRepository, StoreError};

impl IdRepository for MemoryStore {
    fn oid_for_id(&self, key: &IdKey) -> Result<Option<Oid>, StoreError> {
        Ok(self.0.borrow().ids.get(key).cloned())
    }

    fn id_for_oid(&self, holder: &IdHolder, oid: &Oid) -> Result<Option<CallerId>, StoreError> {
        Ok(self
            .0
            .borrow()
            .ids
            .iter()
            .find(|(key, mapped)| {
                key.holder == *holder && key.session.is_none() && *mapped == oid
            })
            .map(|(key, _)| key.id.clone()))
    }

    fn map_id(&self, key: &IdKey, oid: &Oid) -> Result<(), StoreError> {
        self.0.borrow_mut().ids.insert(key.clone(), oid.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use crate::testing::contract;

    #[test]
    fn it_keeps_the_id_repository_contract() {
        contract::id_repository_contract(&MemoryStore::new());
    }
}
