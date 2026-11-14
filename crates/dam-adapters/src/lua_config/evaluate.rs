use std::path::Path;

use dam_application::ConfigError;
use mlua::chunk::ChunkMode;
use mlua::{Lua, Table, Value};

pub(super) fn evaluate(lua: &Lua, path: &Path) -> Result<Option<Table>, ConfigError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ConfigError::Io(format!("{}: {e}", path.display()))),
    };
    let short_name = short_name_lua_never_truncates(path);
    let value: Value = lua
        .load(text)
        .set_name(format!("@{short_name}"))
        .set_mode(ChunkMode::Text)
        .eval()
        .map_err(|e| ConfigError::Invalid(naming_the_whole_path(path, &short_name, &e)))?;
    match value {
        Value::Table(table) => Ok(Some(table)),
        other => Err(ConfigError::Invalid(format!(
            "{} must return a table, and it returned a {}",
            path.display(),
            other.type_name()
        ))),
    }
}

fn short_name_lua_never_truncates(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

fn naming_the_whole_path(path: &Path, short_name: &str, error: &mlua::Error) -> String {
    let message = lua_message(error);
    match message.strip_prefix(short_name) {
        Some(line_and_reason) if line_and_reason.starts_with(':') => {
            format!("{}{line_and_reason}", path.display())
        }
        _ => message,
    }
}

pub(super) fn lua_message(error: &mlua::Error) -> String {
    let text = match error {
        mlua::Error::SyntaxError { message, .. } => message.clone(),
        mlua::Error::RuntimeError(message) => message.clone(),
        mlua::Error::CallbackError { cause, .. } => return lua_message(cause),
        other => other.to_string(),
    };
    match text.split_once("\nstack traceback:") {
        Some((message, _)) => message.to_string(),
        None => text,
    }
}
