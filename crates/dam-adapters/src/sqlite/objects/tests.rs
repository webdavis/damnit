use super::*;

fn failure(code: std::ffi::c_int) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(code),
        Some("database is locked".into()),
    )
}

#[test]
fn a_busy_or_locked_store_is_its_own_outcome() {
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_BUSY)),
        StoreError::Busy(_)
    ));
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_LOCKED)),
        StoreError::Busy(_)
    ));
}

#[test]
fn every_other_store_failure_stays_a_plain_failure() {
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_CORRUPT)),
        StoreError::Failed(_)
    ));
    assert!(matches!(
        sql(failure(rusqlite::ffi::SQLITE_FULL)),
        StoreError::Failed(_)
    ));
    assert!(matches!(
        sql(rusqlite::Error::QueryReturnedNoRows),
        StoreError::Failed(_)
    ));
}
