mod control;
mod listing;
mod locked;
mod new_task;
mod required;
mod rule;
mod template;
mod version;
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
use control::Walked;
use locked::locked_lua;
pub(super) use template::read_project_template;
pub use written::{FileSystemProjectFolders, write_project_file};

const PROJECT_SETTINGS: [&str; 5] = ["path", "minimum_dam", "new_task", "required", "ls"];
const PERSONAL_SETTINGS: [&str; 9] = [
    "owner",
    "remote",
    "token",
    "schema",
    "project_hooks",
    "hook",
    "secrets",
    "sync",
    "except",
];

pub fn read_project_file(file: &Path, this_dam: &str) -> Result<ProjectSettings, ConfigError> {
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
    match control::walked(&table) {
        Walked::Clean => {}
        Walked::HoldsAControlCharacter => {
            return Err(ConfigError::HoldsAControlCharacter {
                file: file.to_path_buf(),
            });
        }
        Walked::TooLarge => return Err(stopped_at_an_error(file)),
    }
    version::needs_no_newer_dam(file, &table, this_dam)?;
    let root = StrictMap::root(&short_name, &table)?;
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
        new_task: new_task::new_task(file, &root)?,
        required: required::required(file, &root)?,
        ls: listing::listing(file, &root)?,
    })
}
