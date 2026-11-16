use std::fmt;

use dam_application::ConfigError;

pub(super) fn project_text(f: &mut fmt::Formatter<'_>, error: &ConfigError) -> fmt::Result {
    match error {
        ConfigError::BelongsInPersonalConfig { key, file } => write!(
            f,
            "{key} belongs in your own config.lua, never in a project file; remove it from {}",
            file.display()
        ),
        ConfigError::HoldsAControlCharacter { file } => write!(
            f,
            "{} holds a control character in a key or a text; remove it",
            file.display()
        ),
        ConfigError::NotAPath { file } => write!(
            f,
            "path in {} is not a path such as work/app/; correct it",
            file.display()
        ),
        _ => Ok(()),
    }
}
