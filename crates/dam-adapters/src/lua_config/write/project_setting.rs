use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use dam_application::{
    ConfigError, SettingChange, SettingKey, SettingValue, SettingsFile, Uneditable,
};

use super::replace::{Holds, replace};
use super::setting_change::lua_literal;
use super::setting_flat::{Flat, flatten, flatten_value, shown_text, under};
use super::setting_path::edit_setting;
use crate::lua_config::evaluate::read_text;
use crate::lua_config::project::{project_setting, project_table, read_project_text};

const OWNER_WRITES: u32 = 0o200;

pub struct ProjectSettingsFile {
    file: PathBuf,
    this_dam: String,
}

impl ProjectSettingsFile {
    pub fn new(file: PathBuf, this_dam: &str) -> ProjectSettingsFile {
        ProjectSettingsFile {
            file,
            this_dam: this_dam.to_string(),
        }
    }

    pub fn file(&self) -> &Path {
        &self.file
    }
}

impl SettingsFile for ProjectSettingsFile {
    fn set(&self, key: &SettingKey, value: &SettingValue) -> Result<SettingChange, ConfigError> {
        self.change(key, Some(value))
    }

    fn unset(&self, key: &SettingKey) -> Result<SettingChange, ConfigError> {
        self.change(key, None)
    }
}

impl ProjectSettingsFile {
    fn change(
        &self,
        key: &SettingKey,
        value: Option<&SettingValue>,
    ) -> Result<SettingChange, ConfigError> {
        let dotted = key.dotted();
        if !project_setting(key) {
            return Err(ConfigError::BelongsInPersonalConfig {
                key: dotted,
                file: self.file.clone(),
            });
        }
        let literal = value.map(lua_literal);
        let by_hand = format!("{dotted} = {}", literal.as_deref().unwrap_or("nil"));
        let current = read_text(&self.file)?.unwrap_or_default();
        let change = SettingChange {
            key: dotted.clone(),
            old: shown_text(&under(&self.flat(&current)?, &dotted)),
            new: value.map(SettingValue::text),
        };
        if value.is_none() && change.old.is_none() {
            return Ok(change);
        }
        if std::fs::metadata(&self.file).is_ok_and(|m| m.permissions().mode() & OWNER_WRITES == 0) {
            return Err(ConfigError::SettingReadOnly {
                key: dotted,
                by_hand,
            });
        }
        let not_editable = |why| ConfigError::SettingNotEditable {
            key: dotted.clone(),
            why,
            by_hand: by_hand.clone(),
        };
        replace(&self.file, Holds::Settings, |current| {
            let current = current.unwrap_or_default();
            let edited = edit_setting(current, &key.path, literal.as_deref())
                .map_err(not_editable)?
                .ok_or_else(|| not_editable(Uneditable::TableBuiltInCode))?;
            read_project_text(&self.file, &edited, &self.this_dam).map_err(|why| {
                ConfigError::ProjectSettingWouldNotLoad {
                    key: dotted.clone(),
                    why: Box::new(why),
                }
            })?;
            let mut expected: Flat = self
                .flat(current)?
                .into_iter()
                .filter(|(name, _)| *name != dotted && !name.starts_with(&format!("{dotted}.")))
                .collect();
            expected.extend(
                value
                    .map(|value| flatten_value(&dotted, value))
                    .into_iter()
                    .flatten(),
            );
            match self.flat(&edited)? == expected {
                true => Ok(edited),
                false => Err(ConfigError::SettingEditWouldBreak {
                    key: dotted.clone(),
                }),
            }
        })?;
        Ok(change)
    }

    fn flat(&self, text: &str) -> Result<Flat, ConfigError> {
        let (_lua, table) = project_table(&self.file, text)?;
        Ok(flatten(Some(&table)))
    }
}
