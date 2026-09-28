mod codec;
mod commits;
mod conflicts;
mod migrations;
mod objects;
mod remote;
mod stage;

use std::fmt;
use std::fs::DirBuilder;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use dam_application::{StoreError, Transactional, UseCaseError};
use rusqlite::{Connection, OpenFlags};

const BUSY_TIMEOUT_MS: u64 = 5000;
const OWNER_ONLY_DIRECTORY: u32 = 0o700;
const OWNER_ONLY_FILE: u32 = 0o600;

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

fn create_owner_only_in_one_call_so_no_symlink_can_be_planted_first(
    path: &Path,
) -> Result<(), OpenError> {
    match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(OWNER_ONLY_FILE)
        .open(path)
    {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            refuse_anything_but_a_regular_file_then_make_it_owner_only(path)
        }
        Err(e) => Err(OpenError::Io(e.to_string())),
    }
}

fn refuse_anything_but_a_regular_file_then_make_it_owner_only(
    path: &Path,
) -> Result<(), OpenError> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| OpenError::Io(e.to_string()))?;
    if !meta.file_type().is_file() {
        return Err(OpenError::Irregular(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(OWNER_ONLY_FILE))
        .map_err(|e| OpenError::Io(e.to_string()))
}

impl SqliteStore {
    pub(super) fn in_savepoint<T, E: From<StoreError>>(
        &self,
        name: &'static str,
        work: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let (open, commit, abort) = if self.conn.is_autocommit() {
            outermost_unit_taking_the_write_lock_at_its_first_statement()
        } else {
            savepoint_nested_in_the_open_unit(name)
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

fn outermost_unit_taking_the_write_lock_at_its_first_statement() -> (String, String, String) {
    (
        "BEGIN IMMEDIATE".to_string(),
        "COMMIT".to_string(),
        "ROLLBACK".to_string(),
    )
}

fn savepoint_nested_in_the_open_unit(name: &str) -> (String, String, String) {
    (
        format!("SAVEPOINT {name}"),
        format!("RELEASE {name}"),
        format!("ROLLBACK TO {name}; RELEASE {name}"),
    )
}

impl Transactional for SqliteStore {
    fn in_transaction(
        &self,
        work: &mut dyn FnMut() -> Result<(), UseCaseError>,
    ) -> Result<(), UseCaseError> {
        self.in_savepoint("dam_unit_of_work", work)
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
