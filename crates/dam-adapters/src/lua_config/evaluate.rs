use std::path::Path;

use dam_application::ConfigError;
use mlua::chunk::ChunkMode;
use mlua::{Lua, Table, Value};

use super::printed::while_the_top_level_runs;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ErrorText {
    QuotesTheFile,
    QuotesWhatItsCodeRaised,
    NeverQuotesTheFile,
}

impl ErrorText {
    pub(super) fn about_what_it_holds(self, path: &Path, error: ConfigError) -> ConfigError {
        match self {
            ErrorText::NeverQuotesTheFile => stopped_at_an_error(path),
            _ => error,
        }
    }
}

pub(super) fn stopped_at_an_error(path: &Path) -> ConfigError {
    ConfigError::StoppedLoading {
        path: path.to_path_buf(),
        line: None,
        reason: None,
    }
}

pub(super) fn read_text(path: &Path) -> Result<Option<String>, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(None),
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Unreadable {
            path: path.to_path_buf(),
            why: e.to_string(),
        }),
    }
}

pub(super) fn evaluate(
    lua: &Lua,
    path: &Path,
    text: Option<&str>,
    error_text: ErrorText,
) -> Result<Option<Table>, ConfigError> {
    let Some(text) = text else {
        return Ok(None);
    };
    match evaluated(lua, path, text, error_text)? {
        Value::Table(table) => Ok(Some(table)),
        other => Err(ConfigError::NotATable {
            path: path.to_path_buf(),
            returned: super::shape::described(&other),
        }),
    }
}

pub(super) fn evaluated(
    lua: &Lua,
    path: &Path,
    text: &str,
    error_text: ErrorText,
) -> Result<Value, ConfigError> {
    let short_name = short_name_lua_never_truncates(path);
    while_the_top_level_runs(lua, error_text, || {
        lua.load(text)
            .set_name(format!("@{short_name}"))
            .set_mode(ChunkMode::Text)
            .eval()
    })
    .map_err(|e| stopped_loading(path, &short_name, &e, error_text))
}

fn stopped_loading(
    path: &Path,
    short_name: &str,
    error: &mlua::Error,
    error_text: ErrorText,
) -> ConfigError {
    let message = lua_message(error);
    let (line, reason) = match line_and_reason(short_name, &message) {
        Some((line, reason)) => (line.parse().ok(), reason.to_string()),
        None => (None, message.clone()),
    };
    let reason = match (error, error_text) {
        (_, ErrorText::QuotesTheFile) => Some(reason),
        (mlua::Error::SyntaxError { .. }, ErrorText::NeverQuotesTheFile) => {
            before_the_first_quote(&without_the_text_lua_stopped_at(reason))
        }
        (mlua::Error::SyntaxError { .. }, _) => Some(without_the_text_lua_stopped_at(reason)),
        (_, ErrorText::QuotesWhatItsCodeRaised) => Some(reason),
        _ => None,
    };
    ConfigError::StoppedLoading {
        path: path.to_path_buf(),
        line,
        reason,
    }
}

pub(super) fn short_name_lua_never_truncates(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

pub(super) fn line_and_reason<'a>(
    short_name: &str,
    message: &'a str,
) -> Option<(&'a str, &'a str)> {
    let (line, reason) = message
        .strip_prefix(short_name)?
        .strip_prefix(':')?
        .split_once(':')?;
    (!line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()))
        .then(|| (line, reason.trim_start()))
}

fn without_the_text_lua_stopped_at(message: String) -> String {
    match message.split_once(" near ") {
        Some((reason, _)) => reason.to_string(),
        None => message,
    }
}

fn before_the_first_quote(message: &str) -> Option<String> {
    let fixed = message
        .split(['\'', '"'])
        .next()
        .unwrap_or_default()
        .trim_end();
    (!fixed.is_empty()).then(|| fixed.to_string())
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
