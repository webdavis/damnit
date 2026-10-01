mod accepted_kinds;
mod codec;
mod commits;
mod conflicts;
mod file;
mod migrations;
mod objects;
mod remote;
mod savepoint;
mod stage;

use std::fmt;
use std::fs::DirBuilder;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

const BUSY_TIMEOUT_MS: u64 = 5000;
const OWNER_ONLY_DIRECTORY: u32 = 0o700;

use file::create_owner_only_in_one_call_so_no_symlink_can_be_planted_first;

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
    pub fn open(path: &Path) -> Result<SqliteStore, OpenError> {
        if let Some(dir) = path.parent() {
            DirBuilder::new()
                .recursive(true)
                .mode(OWNER_ONLY_DIRECTORY)
                .create(dir)
                .map_err(|e| OpenError::Io(e.to_string()))?;
        }
        create_owner_only_in_one_call_so_no_symlink_can_be_planted_first(path)?;
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut conn = Connection::open_with_flags(path, flags)
            .map_err(|e| OpenError::Sqlite(e.to_string()))?;
        prepare(&mut conn, Backing::File)?;
        Ok(SqliteStore { conn })
    }

    pub fn in_memory() -> Result<SqliteStore, OpenError> {
        let mut conn =
            Connection::open_in_memory().map_err(|e| OpenError::Sqlite(e.to_string()))?;
        prepare(&mut conn, Backing::MemoryWhichNeverRunsWal)?;
        Ok(SqliteStore { conn })
    }
}

#[derive(PartialEq)]
enum Backing {
    File,
    MemoryWhichNeverRunsWal,
}

fn prepare(conn: &mut Connection, backing: Backing) -> Result<(), OpenError> {
    conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS))
        .map_err(|e| OpenError::Sqlite(e.to_string()))?;
    let mode: String = conn
        .pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get(0))
        .map_err(|e| OpenError::Sqlite(e.to_string()))?;
    let mode = mode.to_ascii_lowercase();
    if mode != "wal" && !(backing == Backing::MemoryWhichNeverRunsWal && mode == "memory") {
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
    migrations::migrate_above_user_version_in_one_transaction(conn)
        .map_err(|e| OpenError::Sqlite(e.to_string()))
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
mod tests;
