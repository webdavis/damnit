mod locked;
mod written;

use std::path::Path;

use dam_application::{ConfigError, ProjectSettings};
use mlua::Value;
use mlua::chunk::ChunkMode;

use super::evaluate::{
    ErrorText, read_text, short_name_lua_never_truncates, stopped_at_an_error, stopped_loading,
};
use super::shape::described;
use super::strict::StrictMap;
use locked::locked_lua;
pub use written::{FileSystemProjectFolders, write_project_file};

const PROJECT_SETTINGS: [&str; 1] = ["path"];
const PERSONAL_SETTINGS: [&str; 5] = ["owner", "remote", "token", "schema", "project_hooks"];

pub fn read_project_file(file: &Path) -> Result<ProjectSettings, ConfigError> {
    let text = read_text(file)?.unwrap_or_default();
    let lua = locked_lua().map_err(|_| stopped_at_an_error(file))?;
    let short_name = short_name_lua_never_truncates(file);
    let value: Value = lua
        .load(text)
        .set_name(format!("@{short_name}"))
        .set_mode(ChunkMode::Text)
        .eval()
        .map_err(|e| stopped_loading(file, &short_name, &e, ErrorText::NeverQuotesTheFile))?;
    let Value::Table(table) = value else {
        return Err(ConfigError::NotATable {
            path: file.to_path_buf(),
            returned: described(&value),
        });
    };
    let root = StrictMap::root(&short_name, &table)?;
    let holds_a_control_character = |text: &str| text.chars().any(char::is_control);
    if root.entries().any(|(key, value)| {
        holds_a_control_character(key)
            || matches!(value, Value::String(s) if holds_a_control_character(&s.to_string_lossy()))
    }) {
        return Err(ConfigError::HoldsAControlCharacter {
            file: file.to_path_buf(),
        });
    }
    if let Some((key, _)) = root
        .entries()
        .find(|(key, _)| PERSONAL_SETTINGS.contains(key))
    {
        return Err(ConfigError::BelongsInPersonalConfig {
            key: key.to_string(),
            file: file.to_path_buf(),
        });
    }
    root.only(&PROJECT_SETTINGS)?;
    let path = root.string("path")?.ok_or_else(|| root.missing("path"))?;
    let path = dam_domain::Path::parse(&path).map_err(|_| ConfigError::NotAPath {
        file: file.to_path_buf(),
    })?;
    Ok(ProjectSettings {
        path: path.as_str().to_string(),
    })
}
