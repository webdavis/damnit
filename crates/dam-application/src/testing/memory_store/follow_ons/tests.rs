use super::MemoryStore;
use crate::testing::contract;

#[test]
fn it_keeps_the_follow_on_repository_contract() {
    contract::follow_on_repository_contract(&MemoryStore::new());
}
