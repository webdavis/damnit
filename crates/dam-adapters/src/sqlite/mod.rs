mod codec;
mod commits;
mod conflicts;
mod migrations;
mod objects;
mod remote;
mod stage;

use std::fmt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use dam_application::{StoreError, Transactional, UseCaseError};
use rusqlite::{Connection, OpenFlags};

const BUSY_TIMEOUT_MS: u64 = 5000;

#[derive(Debug)]
pub struct SqliteStore {
    conn: Connection,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpenError {
    Io(String),
    Sqlite(String),
    NewerSchema { found: u32, supported: u32 },
    Irregular(String),
}

impl SqliteStore {
    /// Creates the directory and file when missing, mode 0600, WAL, busy
    /// timeout 5 s, migrated to the current version. Refuses a symlink or a
    /// directory at the path.
    pub fn open(path: &Path) -> Result<SqliteStore, OpenError> {
        if let Ok(meta) = std::fs::symlink_metadata(path)
            && (meta.file_type().is_symlink() || meta.is_dir())
        {
            return Err(OpenError::Irregular(format!(
                "{} is not a regular file",
                path.display()
            )));
        }
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| OpenError::Io(e.to_string()))?;
        }
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut conn = Connection::open_with_flags(path, flags)
            .map_err(|e| OpenError::Sqlite(e.to_string()))?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| OpenError::Io(e.to_string()))?;
        prepare(&mut conn, false)?;
        Ok(SqliteStore { conn })
    }

    /// tests
    pub fn in_memory() -> Result<SqliteStore, OpenError> {
        let mut conn =
            Connection::open_in_memory().map_err(|e| OpenError::Sqlite(e.to_string()))?;
        prepare(&mut conn, true)?;
        Ok(SqliteStore { conn })
    }
}

impl SqliteStore {
    /// Runs `work` as one unit of work, so every write it makes lands together
    /// or none of them does.
    ///
    /// The outermost one is `BEGIN IMMEDIATE`, which takes the write lock
    /// before the first statement. A deferred one reads first and upgrades on
    /// its first write, and in WAL mode that upgrade answers `SQLITE_BUSY` at
    /// once without consulting the busy handler, so the connection's timeout
    /// would not cover the read-then-write paths. A nested one is a savepoint,
    /// so an inner unit of work runs the same way whether or not an outer one
    /// is already open.
    pub(super) fn in_savepoint<T, E: From<StoreError>>(
        &self,
        name: &'static str,
        work: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let outermost = self.conn.is_autocommit();
        let (open, commit, abort) = if outermost {
            (
                "BEGIN IMMEDIATE".to_string(),
                "COMMIT".to_string(),
                "ROLLBACK".to_string(),
            )
        } else {
            (
                format!("SAVEPOINT {name}"),
                format!("RELEASE {name}"),
                format!("ROLLBACK TO {name}; RELEASE {name}"),
            )
        };
        self.conn
            .execute_batch(&open)
            .map_err(|e| E::from(objects::sql(e)))?;
        match work() {
            Ok(value) => {
                self.conn
                    .execute_batch(&commit)
                    .map_err(|e| E::from(objects::sql(e)))?;
                Ok(value)
            }
            Err(e) => {
                let _ = self.conn.execute_batch(&abort);
                Err(e)
            }
        }
    }
}

impl Transactional for SqliteStore {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError> {
        self.in_savepoint("dam_unit_of_work", work)
    }
}

/// `allow_memory` accepts the `memory` journal mode SQLite reports for an
/// in-memory connection, which can never run WAL; any other outcome, on any
/// connection, is a failed requirement.
fn prepare(conn: &mut Connection, allow_memory: bool) -> Result<(), OpenError> {
    conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS))
        .map_err(|e| OpenError::Sqlite(e.to_string()))?;
    let mode: String = conn
        .pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get(0))
        .map_err(|e| OpenError::Sqlite(e.to_string()))?;
    let mode = mode.to_ascii_lowercase();
    if mode != "wal" && !(allow_memory && mode == "memory") {
        return Err(OpenError::Sqlite(format!(
            "journal_mode is {mode}, not wal"
        )));
    }
    let found: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| OpenError::Sqlite(e.to_string()))?;
    if found > migrations::VERSION {
        return Err(OpenError::NewerSchema {
            found,
            supported: migrations::VERSION,
        });
    }
    migrations::migrate(conn).map_err(|e| OpenError::Sqlite(e.to_string()))
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenError::Io(s) | OpenError::Sqlite(s) | OpenError::Irregular(s) => f.write_str(s),
            OpenError::NewerSchema { found, supported } => write!(
                f,
                "the store is schema version {found}; this dam supports {supported}. Upgrade dam."
            ),
        }
    }
}

impl std::error::Error for OpenError {}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_application::StageRepository;
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
        // A second writer with its busy handler off, so it answers at once.
        // rusqlite installs a five second one on every connection it opens.
        let other = Connection::open(&path).unwrap();
        other.busy_timeout(std::time::Duration::ZERO).unwrap();
        let mut second_writer = Ok(());
        store
            .in_savepoint::<(), StoreError>("dam_test", || {
                // A read, the way commit_record starts, and no write yet.
                let _: i64 = store
                    .conn
                    .query_row("SELECT COUNT(*) FROM commits", [], |r| r.get(0))
                    .unwrap();
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
    fn staging_reads_and_writes_as_one_unit_that_an_outer_failure_rolls_back() {
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
            // Nested inside an open unit of work, so staging takes the
            // savepoint path rather than opening its own transaction.
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
}
