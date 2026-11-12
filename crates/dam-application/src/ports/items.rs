use dam_domain::{Item, ItemHash, Outcome};

use super::StoreError;

pub trait ItemRepository {
    fn put_item(&self, item: &Item) -> Result<(), StoreError>;
    fn item(&self, hash: &ItemHash) -> Result<Option<Item>, StoreError>;
    fn items(&self) -> Result<Vec<Item>, StoreError>;
    fn settle(&self, hash: &ItemHash, outcome: Outcome) -> Result<(), StoreError>;
}
