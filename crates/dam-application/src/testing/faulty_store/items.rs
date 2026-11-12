use dam_domain::{Item, ItemHash, Outcome};

use super::FaultyStore;
use crate::ports::{ItemRepository, StoreError};

impl ItemRepository for FaultyStore {
    fn put_item(&self, item: &Item) -> Result<(), StoreError> {
        self.write()?;
        self.inner.put_item(item)
    }
    fn item(&self, hash: &ItemHash) -> Result<Option<Item>, StoreError> {
        self.inner.item(hash)
    }
    fn items(&self) -> Result<Vec<Item>, StoreError> {
        self.inner.items()
    }
    fn settle(&self, hash: &ItemHash, outcome: Outcome) -> Result<(), StoreError> {
        self.write()?;
        self.inner.settle(hash, outcome)
    }
}
