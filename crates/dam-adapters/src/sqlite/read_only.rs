use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use super::{BUSY_TIMEOUT_MS, OpenError, SqliteStore, migrations};

pub(super) const SIDE_FILES: [&str; 3] = ["-wal", "-shm", "-journal"];
const HELD_OPEN: [&str; 2] = ["-wal", "-shm"];

pub enum ReadOnly {
    NoStore,
    LeftOver(Vec<PathBuf>),
    Opened(SqliteStore),
}

impl SqliteStore {
    pub const SCHEMA_VERSION: u32 = migrations::VERSION;

    pub fn open_read_only(path: &Path) -> Result<ReadOnly, OpenError> {
        match std::fs::metadata(path) {
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(ReadOnly::NoStore),
            Err(e) => return Err(OpenError::Io(e.to_string())),
            Ok(_) => {}
        }
        let text = path
            .to_str()
            .ok_or_else(|| OpenError::Io(format!("{} is not UTF-8", path.display())))?;
        let beside: Vec<&str> = SIDE_FILES
            .into_iter()
            .filter(|side| Path::new(&format!("{text}{side}")).exists())
            .collect();
        // A read-only open of a WAL store creates its side files and cannot remove them,
        // so a store with none is read as it is, with no lock. The pair alone means another
        // dam holds the store open, and a plain read-only open reads through it.
        let mode = match beside.as_slice() {
            [] => "&immutable=1",
            sides if sides == HELD_OPEN => "",
            sides => {
                let left = sides.iter().map(|side| format!("{text}{side}").into());
                return Ok(ReadOnly::LeftOver(left.collect()));
            }
        };
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(uri(text, mode), flags)
            .map_err(|e| OpenError::Sqlite(e.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_millis(BUSY_TIMEOUT_MS))
            .map_err(|e| OpenError::Sqlite(e.to_string()))?;
        Ok(ReadOnly::Opened(SqliteStore { conn }))
    }

    pub fn schema_version(&self) -> Result<u32, OpenError> {
        self.conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| OpenError::Sqlite(e.to_string()))
    }
}

fn uri(text: &str, mode: &str) -> String {
    let escaped = text
        .replace('%', "%25")
        .replace('?', "%3f")
        .replace('#', "%23");
    format!("file:{escaped}?mode=ro{mode}")
}

#[cfg(test)]
mod tests;
