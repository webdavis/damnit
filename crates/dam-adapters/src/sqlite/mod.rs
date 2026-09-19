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
