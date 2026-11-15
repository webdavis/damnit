use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::read_only::SIDE_FILES;
use super::{OpenError, SqliteStore};

impl SqliteStore {
    pub fn create_if_missing(path: &Path) -> Result<bool, OpenError> {
        for file in
            std::iter::once(path.to_path_buf()).chain(SIDE_FILES.map(|side| beside(path, side)))
        {
            match std::fs::symlink_metadata(&file) {
                Ok(_) => return Ok(false),
                Err(e) if e.kind() == ErrorKind::NotFound => {}
                Err(e) => return Err(OpenError::Io(e.to_string())),
            }
        }
        SqliteStore::open(path).map(|_| true)
    }
}

fn beside(path: &Path, side: &str) -> PathBuf {
    let mut name = OsString::from(path);
    name.push(side);
    PathBuf::from(name)
}
