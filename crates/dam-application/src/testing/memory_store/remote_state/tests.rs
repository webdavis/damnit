use super::MemoryStore;
use crate::testing::contract;

#[test]
fn it_keeps_the_object_repository_contract() {
    contract::object_repository_contract(&MemoryStore::new());
}

#[test]
fn it_keeps_the_stage_repository_contract() {
    contract::stage_repository_contract(&MemoryStore::new());
}

#[test]
fn it_keeps_the_commit_repository_contract() {
    contract::commit_repository_contract(&MemoryStore::new());
}

#[test]
fn it_keeps_the_commit_clears_the_stage_contract() {
    let store = MemoryStore::new();
    contract::commit_clears_the_stage_contract(&store);
}

#[test]
fn it_keeps_the_remote_tracking_repository_contract() {
    contract::remote_tracking_repository_contract(&MemoryStore::new());
}

#[test]
fn it_keeps_the_accepted_kinds_repository_contract() {
    contract::accepted_kinds_repository_contract(&MemoryStore::new());
}

#[test]
fn it_keeps_the_conflict_repository_contract() {
    let store = MemoryStore::new();
    contract::conflict_repository_contract(&store);
}

#[test]
fn it_keeps_the_notice_repository_contract() {
    contract::notice_repository_contract(&MemoryStore::new());
}

#[test]
fn it_keeps_the_transactional_contract() {
    let store = MemoryStore::new();
    contract::transactional_contract(&store);
}

#[test]
fn it_keeps_the_answer_repository_contract() {
    contract::answer_repository_contract(&MemoryStore::new());
}
