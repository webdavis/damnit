use super::*;
use dam_application::{StageRepository, StoreError};
use std::os::unix::fs::PermissionsExt;

#[test]
fn open_creates_a_private_wal_database_at_the_current_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("dam.db");
    let store = SqliteStore::open(&path).unwrap();
    let journal: String = store
        .conn
        .pragma_query_value(None, "journal_mode", |r| r.get(0))
        .unwrap();
    let version: u32 = store
        .conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(journal, "wal");
    assert_eq!(version, migrations::VERSION);
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn the_store_directory_is_created_as_private_as_the_config_directory_beside_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam").join("dam.db");
    SqliteStore::open(&path).unwrap();
    assert_eq!(
        std::fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
}

#[test]
fn a_newer_schema_is_refused_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    let store = SqliteStore::open(&path).unwrap();
    store
        .conn
        .pragma_update(None, "user_version", migrations::VERSION + 1)
        .unwrap();
    drop(store);
    let err = SqliteStore::open(&path).unwrap_err();
    assert_eq!(
        err,
        OpenError::NewerSchema {
            found: migrations::VERSION + 1,
            supported: migrations::VERSION,
        }
    );
}

#[test]
fn a_unit_of_work_holds_the_write_lock_from_its_first_statement() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    let store = SqliteStore::open(&path).unwrap();
    let other = a_second_writer_answering_at_once_rather_than_after_rusqlites_five_seconds(&path);
    let mut second_writer = Ok(());
    store
        .in_savepoint::<(), StoreError>("dam_test", || {
            read_without_writing_the_way_commit_record_starts(&store);
            second_writer = other.execute_batch(
                "BEGIN IMMEDIATE; INSERT INTO commits (id, seq, message, at) \
                 VALUES ('a', 1, 'm', 't'); COMMIT;",
            );
            Ok(())
        })
        .unwrap();
    let err = second_writer.expect_err("the second writer was let in mid unit of work");
    assert_eq!(
        err.sqlite_error_code(),
        Some(rusqlite::ErrorCode::DatabaseBusy),
        "{err}"
    );
}

#[test]
fn staging_inside_an_open_unit_takes_its_savepoint_so_an_outer_failure_rolls_it_back() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(&dir.path().join("dam.db")).unwrap();
    let oid = dam_domain::Oid::generate(&mut |b: &mut [u8]| b.fill(7));
    let change = dam_domain::Change {
        oid: oid.clone(),
        op: dam_domain::Op::Create,
        before: None,
        after: Some(dam_domain::Object::Task(dam_domain::Task::new(oid, "milk"))),
    };
    let outcome = store.in_savepoint::<(), StoreError>("dam_test", || {
        store.stage(change.clone())?;
        assert_eq!(store.staged().unwrap().len(), 1);
        Err(StoreError::Failed("the caller gave up".into()))
    });
    assert!(outcome.is_err());
    assert_eq!(
        store.staged().unwrap().len(),
        0,
        "the staged row survived a rolled back unit of work"
    );
}

#[test]
fn a_symlink_at_the_path_is_refused_rather_than_followed() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = dir.path().join("elsewhere.db");
    std::fs::write(&elsewhere, b"").unwrap();
    let path = dir.path().join("dam.db");
    std::os::unix::fs::symlink(&elsewhere, &path).unwrap();
    assert!(matches!(
        SqliteStore::open(&path),
        Err(OpenError::Irregular(_))
    ));
}

#[test]
fn a_directory_at_the_path_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dam.db");
    std::fs::create_dir(&path).unwrap();
    assert!(matches!(
        SqliteStore::open(&path),
        Err(OpenError::Irregular(_))
    ));
}

#[test]
fn a_symlink_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("real.db");
    std::fs::write(&target, b"").unwrap();
    let link = dir.path().join("dam.db");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(matches!(
        SqliteStore::open(&link),
        Err(OpenError::Irregular(_))
    ));
}

fn a_second_writer_answering_at_once_rather_than_after_rusqlites_five_seconds(
    path: &std::path::Path,
) -> Connection {
    let other = Connection::open(path).unwrap();
    other.busy_timeout(std::time::Duration::ZERO).unwrap();
    other
}

fn read_without_writing_the_way_commit_record_starts(store: &SqliteStore) {
    let _: i64 = store
        .conn
        .query_row("SELECT COUNT(*) FROM commits", [], |r| r.get(0))
        .unwrap();
}
