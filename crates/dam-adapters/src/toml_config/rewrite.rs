use std::ffi::OsString;
use std::fs::{DirBuilder, OpenOptions, Permissions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path as FsPath, PathBuf};

use super::ConfigError;

fn current_text(path: &FsPath) -> Result<String, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(t) => Ok(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(io_error(e)),
    }
}

pub(super) fn ready_for_a_new_table(mut text: String) -> String {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text
}

pub(super) fn edit(
    path: &FsPath,
    change: impl FnOnce(String) -> Result<String, ConfigError>,
) -> Result<Option<String>, ConfigError> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent_of(path))
        .map_err(io_error)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(0o600)
        .open(lock_path(path))
        .map_err(io_error)?;
    lock.lock().map_err(io_error)?;
    let text = change(current_text(path)?)?;
    replace_atomically_readable_by_its_owner_alone(path, &text)
}

fn lock_path(path: &FsPath) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".lock");
    PathBuf::from(name)
}

pub(super) fn parent_of(path: &FsPath) -> PathBuf {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn io_error(e: std::io::Error) -> ConfigError {
    ConfigError::Io(e.to_string())
}

fn replace_atomically_readable_by_its_owner_alone(
    path: &FsPath,
    text: &str,
) -> Result<Option<String>, ConfigError> {
    let mode_before = match std::fs::metadata(path) {
        Ok(meta) => meta.permissions().mode() & 0o777,
        Err(_) => 0,
    };
    let mut file = tempfile::NamedTempFile::new_in(parent_of(path)).map_err(io_error)?;
    file.as_file()
        .set_permissions(Permissions::from_mode(0o600))
        .map_err(io_error)?;
    file.write_all(text.as_bytes()).map_err(io_error)?;
    file.as_file().sync_all().map_err(io_error)?;
    file.persist(path).map_err(|e| io_error(e.error))?;
    if mode_before & 0o077 == 0 {
        return Ok(None);
    }
    Ok(Some(format!(
        "the config was mode {mode_before:o}; writing it 600, it can hold a credential"
    )))
}
