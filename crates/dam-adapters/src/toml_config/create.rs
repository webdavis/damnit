use std::fs::DirBuilder;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::DirBuilderExt;
use std::path::Path as FsPath;

use super::rewrite::parent_of;

pub(super) const TEMPLATE: &str = include_str!("template.toml");

pub(super) fn create_if_missing(path: &FsPath) -> std::io::Result<bool> {
    if path.try_exists()? {
        return Ok(false);
    }
    let dir = parent_of(path);
    DirBuilder::new().recursive(true).mode(0o700).create(&dir)?;
    let mut file = tempfile::NamedTempFile::new_in(&dir)?;
    file.write_all(TEMPLATE.as_bytes())?;
    file.as_file().sync_all()?;
    match file.persist_noclobber(path) {
        Ok(_) => Ok(true),
        Err(e) if e.error.kind() == ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e.error),
    }
}

#[cfg(test)]
mod tests;
