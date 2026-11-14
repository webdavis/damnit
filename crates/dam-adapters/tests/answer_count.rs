use dam_adapters::SqliteStore;
use dam_application::{AnswerRepository, RemoteName};
use dam_domain::Oid;

#[test]
fn the_count_of_answers_survives_reopening_the_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    let todoist = RemoteName("todoist".into());
    let ada = Oid::generate(&mut |b: &mut [u8]| b.fill(1));
    let store = SqliteStore::open(&path).unwrap();
    store.count_answer(&todoist, &ada).unwrap();
    drop(store);
    let reopened = SqliteStore::open(&path).unwrap();
    assert_eq!(reopened.answers(&todoist, &ada).unwrap(), 1);
}
