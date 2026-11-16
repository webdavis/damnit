mod categories;
mod control;
mod done_children;
mod filters;
mod listing;
mod locked;
mod new_task;
mod required;
mod rule;
mod sections;
mod template;
mod version;
mod written;

use std::path::Path;

use dam_application::{ConfigError, ProjectSettings, SettingKey};
use mlua::chunk::ChunkMode;
use mlua::{Lua, Table, Value};

use super::evaluate::{
    ErrorText, read_text, short_name_lua_never_truncates, stopped_at_an_error, stopped_loading,
};
use super::shape::described;
use super::strict::StrictMap;
use control::Walked;
use locked::locked_lua;
pub(super) use template::read_project_template;
pub use written::{FileSystemProjectFolders, write_project_file};

pub(in crate::lua_config) const PROJECT_SETTINGS: [&str; 10] = [
    "path",
    "minimum_dam",
    "new_task",
    "required",
    "list",
    "filter",
    "category",
    "sections",
    "branch_label",
    "done_children",
];
const PERSONAL_SETTINGS: [&str; 11] = [
    "owner",
    "remote",
    "token",
    "schema",
    "project_hooks",
    "hook",
    "secrets",
    "sync",
    "except",
    "project",
    "project.hooks",
];

pub fn project_setting(key: &SettingKey) -> bool {
    key.path
        .first()
        .is_some_and(|top| PROJECT_SETTINGS.contains(&top.as_str()))
}

pub fn read_project_file(file: &Path, this_dam: &str) -> Result<ProjectSettings, ConfigError> {
    let text = read_text(file)?.unwrap_or_default();
    read_project_text(file, &text, this_dam)
}

pub(in crate::lua_config) fn project_table(
    file: &Path,
    text: &str,
) -> Result<(Lua, Table), ConfigError> {
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
    Ok((lua, table))
}

pub(in crate::lua_config) fn read_project_text(
    file: &Path,
    text: &str,
    this_dam: &str,
) -> Result<ProjectSettings, ConfigError> {
    let (_lua, table) = project_table(file, text)?;
    let short_name = short_name_lua_never_truncates(file);
    match control::walked(&table) {
        Walked::Clean => {}
        Walked::HoldsAControlCharacter => {
            return Err(ConfigError::HoldsAControlCharacter {
                file: file.to_path_buf(),
            });
        }
        Walked::TooLarge => return Err(stopped_at_an_error(file)),
    }
    let minimum_dam = version::needs_no_newer_dam(file, &table, this_dam)?;
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
    let categories = categories::categories(file, &root)?;
    let list = listing::listing(file, &root, &categories)?;
    root.only(&PROJECT_SETTINGS)?;
    let path = root.string("path")?.ok_or_else(|| root.missing("path"))?;
    let path = dam_domain::Path::parse(&path).map_err(|_| ConfigError::NotAPath {
        file: file.to_path_buf(),
    })?;
    Ok(ProjectSettings {
        path: path.as_str().to_string(),
        minimum_dam,
        new_task: new_task::new_task(file, &root)?,
        required: required::required(file, &root)?,
        list,
        filters: filters::filters(file, &root, &categories)?,
        categories,
        sections: sections::sections(file, &root)?,
        branch_label: root
            .bool("branch_label")
            .map_err(|e| rule::misshapen(file, "branch_label")(e.into()))?
            .unwrap_or(false),
        done_children: done_children::done_children(file, &root)?,
    })
}
