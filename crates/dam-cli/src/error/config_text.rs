use std::fmt;

use dam_application::ConfigError;

use super::edit_text::{ByHandClause, ByHandText, cannot_edit, named, would_break};
use super::misread_text::{MisreadText, PlaceText};
use super::setting_text::setting_reason;
use filter::filter_text;
use hook_moved::hook_moved;
use project::project_text;

mod filter;
mod hook_moved;
mod project;

pub(crate) fn config_text(error: &ConfigError) -> String {
    ConfigText(error).to_string()
}

pub(super) struct ConfigText<'a>(pub(super) &'a ConfigError);

pub(crate) struct ConfigReason<'a>(pub(crate) &'a ConfigError);

impl fmt::Display for ConfigText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let heading = match self.0 {
            ConfigError::Unreadable { .. } => "reading config",
            ConfigError::Unwritable { .. } => "writing config",
            _ => "config",
        };
        write!(f, "{heading}: {}", ConfigReason(self.0))
    }
}

impl fmt::Display for ConfigReason<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ConfigError::NotLua(path) => write!(
                f,
                "{} does not end in .lua; the config is config.lua now",
                path.display()
            ),
            ConfigError::Unreadable { path, why } | ConfigError::Unwritable { path, why } => {
                write!(f, "{}: {why}", path.display())
            }
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
            ConfigError::HookMoved { config, events } => hook_moved(f, config, events),
            ConfigError::UnknownHookFile { file } => write!(
                f,
                "{file} names no event; a hook file is named task_new.lua, task_changed.lua, \
                 task_done.lua or task_pulled.lua"
            ),
            ConfigError::NotAHook { file, returned } => write!(
                f,
                "{file} must return a function, or a table holding one as run, and it returned \
                 {returned}"
            ),
            ConfigError::Misread(misread) => write!(f, "{}", MisreadText(misread)),
            ConfigError::BelongsInSecrets { key, at } => {
                write!(f, "{key} belongs in secrets.lua {}", PlaceText(at))
            }
            ConfigError::BelongsInPersonalConfig { .. }
            | ConfigError::HoldsAControlCharacter { .. }
            | ConfigError::NotAPath { .. }
            | ConfigError::NeedsANewerDam { .. }
            | ConfigError::ProjectRule { .. } => project_text(f, self.0),
            ConfigError::QueryText { .. }
            | ConfigError::FilterTable { .. }
            | ConfigError::FilterReachesItself { .. } => filter_text(f, self.0),
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
            ConfigError::ReadOnly { table, by_hand } => {
                let (file, _) = named(*table);
                write!(
                    f,
                    "{file} is read-only; make it writable, or {}",
                    ByHandClause(by_hand)
                )
            }
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
            other => setting_reason(f, other),
        }
    }
}

#[cfg(test)]
mod tests;
