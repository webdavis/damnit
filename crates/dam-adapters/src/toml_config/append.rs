use std::fs::{DirBuilder, Permissions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path as FsPath, PathBuf};

use toml::Value;

use super::{ConfigError, parse_config};

pub(super) fn append_remote(
    path: &FsPath,
    name: &str,
    url: &str,
) -> Result<Option<String>, ConfigError> {
    let mut text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(ConfigError::Io(e.to_string())),
    };
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&format!(
        "[remote.{name}]\nurl = {}\n",
        Value::String(url.to_string())
    ));
    readable_with_the_new_remote(&text, name)?;
    let dir = parent_of(path);
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&dir)
        .map_err(io_error)?;
    replace_atomically_readable_by_its_owner_alone(path, &text)
}

fn readable_with_the_new_remote(text: &str, name: &str) -> Result<(), ConfigError> {
    parse_config(text)?.remote(name).map(|_| ()).ok_or_else(|| {
        ConfigError::Invalid(format!("the new config would not hold remote {name:?}"))
    })
}

fn parent_of(path: &FsPath) -> PathBuf {
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

#[cfg(test)]
mod tests;
