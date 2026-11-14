use std::fs::{DirBuilder, File, Permissions};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use dam_application::{ConfigError, SecretsTightened};

use crate::lua_config::evaluate::read_text;

const OWNER_ONLY: u32 = 0o600;
const DEFAULT_MODE: u32 = 0o666;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Holds {
    Settings,
    Secrets,
}

pub(super) fn replace(
    path: &Path,
    holds: Holds,
    change: impl FnOnce(Option<&str>) -> Result<String, ConfigError>,
) -> Result<Option<SecretsTightened>, ConfigError> {
    let io_error = |error: std::io::Error| ConfigError::Unwritable {
        path: path.to_path_buf(),
        why: system_reason(&error),
    };
    let path = &resolved(path)?;
    let dir = folder_of(path);
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .map_err(io_error)?;
    let folder = File::open(&dir).map_err(io_error)?;
    folder.lock().map_err(io_error)?;
    let text = change(read_text(path)?.as_deref())?;
    let mode_before = std::fs::metadata(path)
        .ok()
        .map(|meta| meta.permissions().mode() & 0o777);
    let mut file = tempfile::Builder::new()
        .permissions(Permissions::from_mode(match holds {
            Holds::Secrets => OWNER_ONLY,
            Holds::Settings => mode_before.unwrap_or(DEFAULT_MODE),
        }))
        .tempfile_in(&dir)
        .map_err(io_error)?;
    if let (Holds::Settings, Some(mode)) = (holds, mode_before) {
        let exact = Permissions::from_mode(mode);
        file.as_file().set_permissions(exact).map_err(io_error)?;
    }
    file.write_all(text.as_bytes()).map_err(io_error)?;
    file.as_file().sync_all().map_err(io_error)?;
    file.persist(path).map_err(|e| io_error(e.error))?;
    folder.sync_all().map_err(io_error)?;
    Ok(match (holds, mode_before) {
        (Holds::Secrets, Some(mode)) if mode & 0o077 != 0 => Some(SecretsTightened { was: mode }),
        _ => None,
    })
}

fn system_reason(error: &std::io::Error) -> String {
    match error.raw_os_error() {
        Some(code) => std::io::Error::from_raw_os_error(code).to_string(),
        None => error.kind().to_string(),
    }
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
        Err(e) => Err(ConfigError::Unreadable {
            path: path.to_path_buf(),
            why: e.to_string(),
        }),
    }
}
