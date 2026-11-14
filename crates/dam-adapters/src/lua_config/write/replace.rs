use std::ffi::OsString;
use std::fs::{DirBuilder, File, OpenOptions, Permissions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use dam_application::ConfigError;

use crate::lua_config::evaluate::read_text;

const OWNER_ONLY: u32 = 0o600;
const DEFAULT_MODE: u32 = 0o666;

pub(super) fn replace(
    path: &Path,
    change: impl FnOnce(Option<&str>) -> Result<String, ConfigError>,
) -> Result<(), ConfigError> {
    let path = &resolved(path)?;
    let dir = folder_of(path);
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .map_err(io_error)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(OWNER_ONLY)
        .open(lock_path(path))
        .map_err(io_error)?;
    lock.lock().map_err(io_error)?;
    let text = change(read_text(path)?.as_deref())?;
    let mode_before = std::fs::metadata(path)
        .ok()
        .map(|meta| meta.permissions().mode() & 0o777);
    let mut file = tempfile::Builder::new()
        .permissions(Permissions::from_mode(mode_before.unwrap_or(DEFAULT_MODE)))
        .tempfile_in(&dir)
        .map_err(io_error)?;
    if let Some(mode) = mode_before {
        let exact = Permissions::from_mode(mode);
        file.as_file().set_permissions(exact).map_err(io_error)?;
    }
    file.write_all(text.as_bytes()).map_err(io_error)?;
    file.as_file().sync_all().map_err(io_error)?;
    file.persist(path).map_err(|e| io_error(e.error))?;
    File::open(&dir)
        .and_then(|folder| folder.sync_all())
        .map_err(io_error)?;
    Ok(())
}

pub(in crate::lua_config) fn folder_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn resolved(path: &Path) -> Result<PathBuf, ConfigError> {
    match std::fs::canonicalize(path) {
        Ok(target) => Ok(target),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(e) => Err(io_error(e)),
    }
}

fn lock_path(path: &Path) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".lock");
    PathBuf::from(name)
}

fn io_error(error: std::io::Error) -> ConfigError {
    ConfigError::Io(error.to_string())
}
