use crate::testing::{MemoryStore, contract};

#[test]
fn it_keeps_the_day_repository_contract() {
    contract::day_repository_contract(&MemoryStore::new());
}
