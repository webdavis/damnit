use std::collections::BTreeSet;
use std::path::Path;

use dam_application::{CommitRepository, RemoteName};
use dam_domain::{CommitId, CommitRecord};

use super::super::{ReadOnly, SqliteStore, migrations};

fn opened(path: &Path) -> SqliteStore {
    match SqliteStore::open_read_only(path).unwrap() {
        ReadOnly::Opened(store) => store,
        _ => panic!("{} did not open", path.display()),
    }
}

fn listing(folder: &Path) -> BTreeSet<String> {
    std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

fn empty_commit() -> CommitRecord {
    CommitRecord {
        id: CommitId::generate(&mut |x| x.fill(7)),
        message: "m".into(),
        pulled: false,
        at: jiff::Timestamp::UNIX_EPOCH,
        changes: Vec::new(),
    }
}

fn store_at_version(path: &Path, version: u32) {
    let store = SqliteStore::open(path).unwrap();
    store
        .conn
        .pragma_update(None, "user_version", version)
        .unwrap();
}

#[test]
fn no_file_reads_as_none_and_creates_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam").join("dam.db");
    assert!(matches!(
        SqliteStore::open_read_only(&path).unwrap(),
        ReadOnly::NoStore
    ));
    assert!(listing(dir.path()).is_empty());
}

#[test]
fn a_closed_store_reads_its_version_and_leaves_no_side_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    store_at_version(&path, migrations::VERSION);
    let before = listing(dir.path());
    assert_eq!(before, BTreeSet::from(["dam.db".to_string()]));
    let store = opened(&path);
    assert_eq!(store.schema_version().unwrap(), SqliteStore::SCHEMA_VERSION);
    assert!(
        store
            .unpushed(&RemoteName("todoist".into()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(listing(dir.path()), before);
    drop(store);
    assert_eq!(listing(dir.path()), before);
}

#[test]
fn an_older_store_reads_its_own_version_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    store_at_version(&path, 11);
    let store = opened(&path);
    assert_eq!(store.schema_version().unwrap(), 11);
    drop(store);
    let reopened = opened(&path);
    assert_eq!(reopened.schema_version().unwrap(), 11);
}

#[test]
fn a_store_another_dam_holds_open_reads_what_it_wrote() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    let writer = SqliteStore::open(&path).unwrap();
    writer.commit(&empty_commit()).unwrap();
    let held = listing(dir.path());
    let store = opened(&path);
    assert_eq!(store.schema_version().unwrap(), SqliteStore::SCHEMA_VERSION);
    assert_eq!(store.log().unwrap().len(), 1);
    assert_eq!(listing(dir.path()), held);
}

#[test]
fn a_file_that_is_not_a_store_fails_to_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    std::fs::write(&path, "not a database, only text ".repeat(40)).unwrap();
    let store = opened(&path);
    assert!(store.schema_version().is_err());
    assert_eq!(listing(dir.path()), BTreeSet::from(["dam.db".to_string()]));
}

#[test]
fn a_path_holding_uri_characters_still_opens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("odd?name#100%.db");
    store_at_version(&path, migrations::VERSION);
    let store = opened(&path);
    assert_eq!(store.schema_version().unwrap(), SqliteStore::SCHEMA_VERSION);
}

fn left_beside(path: &Path, suffixes: &[&str]) -> Vec<std::path::PathBuf> {
    suffixes
        .iter()
        .map(|suffix| {
            let side = std::path::PathBuf::from(format!("{}{suffix}", path.display()));
            std::fs::write(&side, b"").unwrap();
            side
        })
        .collect()
}

#[test]
fn a_store_with_any_side_file_but_the_pair_is_not_read_and_names_what_is_left() {
    for suffixes in [
        &["-wal"][..],
        &["-shm"],
        &["-journal"],
        &["-wal", "-shm", "-journal"],
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dam.db");
        store_at_version(&path, migrations::VERSION);
        let left = left_beside(&path, suffixes);
        let before = listing(dir.path());
        let opened = SqliteStore::open_read_only(&path).unwrap();
        assert!(
            matches!(&opened, ReadOnly::LeftOver(files) if *files == left),
            "{suffixes:?}"
        );
        assert_eq!(listing(dir.path()), before, "{suffixes:?}");
    }
}
