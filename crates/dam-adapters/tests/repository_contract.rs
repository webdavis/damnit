//! The durable store answers the same repository contract the in-memory
//! double does, so a use case proved against one holds on the other.

use dam_adapters::SqliteStore;
use dam_application::testing::contract;

fn store() -> SqliteStore {
    SqliteStore::in_memory().unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn it_keeps_the_object_repository_contract() {
    contract::object_repository_contract(&store());
}

#[test]
fn it_keeps_the_stage_repository_contract() {
    contract::stage_repository_contract(&store());
}

#[test]
fn it_keeps_the_commit_repository_contract() {
    contract::commit_repository_contract(&store());
}

#[test]
fn it_keeps_the_remote_tracking_repository_contract() {
    contract::remote_tracking_repository_contract(&store());
}

#[test]
fn it_keeps_the_conflict_repository_contract() {
    let store = store();
    contract::conflict_repository_contract(&store, &store);
}

#[test]
fn it_keeps_the_notice_repository_contract() {
    contract::notice_repository_contract(&store());
}
