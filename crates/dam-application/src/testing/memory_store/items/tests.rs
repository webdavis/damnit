use super::MemoryStore;
use crate::testing::contract;

#[test]
fn it_keeps_the_item_repository_contract() {
    contract::item_repository_contract(&MemoryStore::new());
}
