use std::path::Path;

use dam_application::ConfigError;
use mlua::chunk::ChunkMode;
use mlua::{Lua, Table, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ErrorText {
    QuotesTheFile,
    NeverQuotesTheFile,
}

pub(super) fn evaluate(
    lua: &Lua,
    path: &Path,
    error_text: ErrorText,
) -> Result<Option<Table>, ConfigError> {
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
        .map_err(|e| {
            let message = naming_the_whole_path(path, &short_name, &e);
            ConfigError::Invalid(match (&e, error_text) {
                (_, ErrorText::QuotesTheFile) => message,
                (mlua::Error::SyntaxError { .. }, _) => without_the_text_lua_stopped_at(message),
                _ => only_where_it_stopped(path, &short_name, &e),
            })
        })?;
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

fn only_where_it_stopped(path: &Path, short_name: &str, error: &mlua::Error) -> String {
    let message = lua_message(error);
    let line = message
        .strip_prefix(short_name)
        .and_then(|rest| rest.strip_prefix(':'))
        .and_then(|rest| rest.split_once(':'))
        .map(|(line, _)| line)
        .filter(|line| !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()));
    match line {
        Some(line) => format!("{}:{line}: loading stopped at an error", path.display()),
        None => format!("{}: loading stopped at an error", path.display()),
    }
}

fn without_the_text_lua_stopped_at(message: String) -> String {
    match message.split_once(" near ") {
        Some((reason, _)) => reason.to_string(),
        None => message,
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
