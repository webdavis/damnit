use super::MemoryStore;
use crate::testing::contract;

#[test]
fn it_keeps_the_id_repository_contract() {
    contract::id_repository_contract(&MemoryStore::new());
}
