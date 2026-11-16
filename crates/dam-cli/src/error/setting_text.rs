use std::fmt;

use dam_application::{ConfigError, Uneditable};

pub(super) fn setting_reason(f: &mut fmt::Formatter<'_>, error: &ConfigError) -> fmt::Result {
    match error {
        ConfigError::SettingNotEditable { key, why, by_hand } => {
            let built = match why {
                Uneditable::NotEditable => "config.lua has Lua that dam cannot edit",
                Uneditable::ReturnsBuiltInCode => "config.lua returns a table built in code",
                Uneditable::TableBuiltInCode => {
                    "a table that holds it in config.lua is built in code"
                }
            };
            write!(
                f,
                "cannot write {key}: {built}; change config.lua by hand: {by_hand}"
            )
        }
        ConfigError::SettingEditWouldBreak { key } => write!(
            f,
            "cannot write {key}: config.lua would read differently in another setting; nothing was changed"
        ),
        ConfigError::SettingReadOnly { key, by_hand } => write!(
            f,
            "cannot write {key}: config.lua is read-only; make it writable, or change it by hand: {by_hand}"
        ),
        ConfigError::ProjectSettingWouldNotLoad { key, .. } => write!(
            f,
            "cannot set {key}: .dam.lua would not load with that value, so nothing was changed; docs/lua.md lists what {key} takes"
        ),
        _ => Ok(()),
    }
}
