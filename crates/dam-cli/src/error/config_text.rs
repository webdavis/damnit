use std::fmt;

use dam_application::ConfigError;

use super::edit_text::{ByHandText, cannot_edit, named, would_break};
use super::misread_text::{MisreadText, PlaceText};

pub(super) struct ConfigText<'a>(pub(super) &'a ConfigError);

impl fmt::Display for ConfigText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let heading = match self.0 {
            ConfigError::Unreadable { .. } => "reading config",
            _ => "config",
        };
        write!(f, "{heading}: ")?;
        match self.0 {
            ConfigError::NotLua(path) => write!(
                f,
                "{} does not end in .lua; the config is config.lua now",
                path.display()
            ),
            ConfigError::Unreadable { path, why } => write!(f, "{}: {why}", path.display()),
            ConfigError::StoppedLoading { path, line, reason } => {
                write!(f, "{}", path.display())?;
                if let Some(line) = line {
                    write!(f, ":{line}")?;
                }
                let reason = reason.as_deref().unwrap_or("loading stopped at an error");
                write!(f, ": {reason}")
            }
            ConfigError::NotATable { path, returned } => write!(
                f,
                "{} must return a table, and it returned {returned}",
                path.display()
            ),
            ConfigError::NotNamedSettings { file } => {
                write!(f, "{file} must return a table of named settings")
            }
            ConfigError::Misread(misread) => write!(f, "{}", MisreadText(misread)),
            ConfigError::BelongsInSecrets { key, at } => {
                write!(f, "{key} belongs in secrets.lua {}", PlaceText(at))
            }
            ConfigError::BadRemoteName(why) => write!(f, "{why}"),
            ConfigError::BadCategories(why) => write!(f, "category: {why:?}"),
            ConfigError::UndeclaredRemote { remote, config } => write!(
                f,
                "remote {remote} in secrets.lua is not declared in {config}"
            ),
            ConfigError::CannotEdit {
                table,
                why,
                by_hand,
            } => cannot_edit(f, *table, *why, by_hand),
            ConfigError::AlreadyHeld { table, entry } => {
                let (file, table) = named(*table);
                write!(f, "{file} already holds {table} {entry}")
            }
            ConfigError::NotHeld { table, entry } => {
                let (file, table) = named(*table);
                write!(f, "{file} holds no {table} {entry}")
            }
            ConfigError::HeldUnderUneditableKey {
                table,
                entry,
                by_hand,
            } => {
                let (file, table) = named(*table);
                write!(
                    f,
                    "{file} holds {table} {entry} under a key dam cannot edit. {}",
                    ByHandText(by_hand)
                )
            }
            ConfigError::EditWouldBreak {
                table,
                entry,
                fault,
            } => would_break(f, *table, entry, *fault),
        }
    }
}

#[cfg(test)]
mod tests;
