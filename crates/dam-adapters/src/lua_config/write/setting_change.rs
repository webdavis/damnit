use std::os::unix::fs::PermissionsExt;

use dam_application::{ConfigError, SettingChange, SettingKey, SettingValue, SettingsFile};

use super::lua_text::{list_literal, string_literal};
use super::replace::{Holds, replace};
use super::setting_flat::{Flat, flatten, flatten_value, shown_text, under};
use super::setting_path::edit_setting;
use crate::lua_config::LuaConfigFile;
use crate::lua_config::evaluate::read_text;

const OWNER_WRITES: u32 = 0o200;
const MISSING: &str = "return {\n}\n";

impl SettingsFile for LuaConfigFile {
    fn set(&self, key: &SettingKey, value: &SettingValue) -> Result<SettingChange, ConfigError> {
        self.change_setting(key, Some(value))
    }

    fn unset(&self, key: &SettingKey) -> Result<SettingChange, ConfigError> {
        self.change_setting(key, None)
    }
}

impl LuaConfigFile {
    fn change_setting(
        &self,
        key: &SettingKey,
        value: Option<&SettingValue>,
    ) -> Result<SettingChange, ConfigError> {
        let dotted = key.dotted();
        let literal = value.map(lua_literal);
        let by_hand = format!("{dotted} = {}", literal.as_deref().unwrap_or("nil"));
        let path = self.config_path();
        let current = read_text(path)?;
        let old = shown_text(&under(&self.flat(current.as_deref())?, &dotted));
        let change = SettingChange {
            key: dotted.clone(),
            old,
            new: value.map(SettingValue::text),
        };
        if value.is_none() && change.old.is_none() {
            return Ok(change);
        }
        if std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & OWNER_WRITES == 0) {
            return Err(ConfigError::SettingReadOnly {
                key: dotted,
                by_hand,
            });
        }
        replace(path, Holds::Settings, |current| {
            let before = self.flat(current)?;
            let edited = edit_setting(current.unwrap_or(MISSING), &key.path, literal.as_deref())
                .map_err(|why| ConfigError::SettingNotEditable {
                    key: dotted.clone(),
                    why,
                    by_hand: by_hand.clone(),
                })?
                .ok_or_else(|| ConfigError::SettingNotEditable {
                    key: dotted.clone(),
                    why: dam_application::Uneditable::TableBuiltInCode,
                    by_hand: by_hand.clone(),
                })?;
            let mut expected: Flat = before
                .into_iter()
                .filter(|(name, _)| *name != dotted && !name.starts_with(&format!("{dotted}.")))
                .collect();
            if let Some(value) = value {
                expected.extend(flatten_value(&dotted, value));
            }
            match self.flat(Some(&edited))? == expected {
                true => Ok(edited),
                false => Err(ConfigError::SettingEditWouldBreak {
                    key: dotted.clone(),
                }),
            }
        })?;
        Ok(change)
    }

    fn flat(&self, text: Option<&str>) -> Result<Flat, ConfigError> {
        let kept = self.settings_kept(text)?;
        Ok(flatten(kept.settings.as_ref()))
    }
}

fn lua_literal(value: &SettingValue) -> String {
    match value {
        SettingValue::Text(text) => string_literal(text),
        SettingValue::Flag(flag) => flag.to_string(),
        SettingValue::Count(count) => count.to_string(),
        SettingValue::List(items) => list_literal(items),
    }
}
