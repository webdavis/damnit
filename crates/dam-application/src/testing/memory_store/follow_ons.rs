use dam_domain::ItemHash;

use super::MemoryStore;
use crate::ports::{FollowOn, FollowOnRepository, StoreError};

impl FollowOnRepository for MemoryStore {
    fn keep_follow_ons(&self, item: &ItemHash, follow_ons: &[FollowOn]) -> Result<(), StoreError> {
        self.0
            .borrow_mut()
            .follow_ons
            .insert(item.clone(), follow_ons.to_vec());
        Ok(())
    }

    fn follow_ons(&self, item: &ItemHash) -> Result<Vec<FollowOn>, StoreError> {
        Ok(self
            .0
            .borrow()
            .follow_ons
            .get(item)
            .cloned()
            .unwrap_or_default())
    }

    fn forget_follow_ons(&self, item: &ItemHash) -> Result<(), StoreError> {
        self.0.borrow_mut().follow_ons.remove(item);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
