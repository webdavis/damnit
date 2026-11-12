use dam_domain::{Item, ItemHash, Outcome};

use super::MemoryStore;
use crate::ports::{ItemRepository, StoreError};

impl ItemRepository for MemoryStore {
    fn put_item(&self, item: &Item) -> Result<(), StoreError> {
        self.0.borrow_mut().items.push(item.clone());
        Ok(())
    }

    fn item(&self, hash: &ItemHash) -> Result<Option<Item>, StoreError> {
        Ok(self
            .0
            .borrow()
            .items
            .iter()
            .find(|i| i.hash == *hash)
            .cloned())
    }

    fn items(&self) -> Result<Vec<Item>, StoreError> {
        Ok(self.0.borrow().items.clone())
    }

    fn settle(&self, hash: &ItemHash, outcome: Outcome) -> Result<(), StoreError> {
        for item in self.0.borrow_mut().items.iter_mut() {
            if item.hash == *hash {
                item.outcome = outcome;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use crate::testing::contract;

    #[test]
    fn it_keeps_the_item_repository_contract() {
        contract::item_repository_contract(&MemoryStore::new());
    }
}
