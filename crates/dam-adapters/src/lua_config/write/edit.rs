use std::collections::BTreeSet;
use std::path::PathBuf;

use dam_application::ConfigError;

use super::insert::Indent;
use super::replace::{Holds, replace};
use super::section::Section;
use crate::lua_config::LuaConfigFile;

pub(in crate::lua_config) enum Change<'a, R> {
    Add {
        entry: &'a dyn Fn(&Indent) -> String,
        as_written: &'a dyn Fn(&R) -> bool,
    },
    Remove,
}

pub(in crate::lua_config) struct EditedFile<'a, R> {
    pub(super) path: PathBuf,
    pub(super) holds: Holds,
    pub(super) file: &'static str,
    pub(super) table: &'static str,
    pub(super) missing: &'static str,
    pub(super) config: &'a LuaConfigFile,
    pub(super) read: fn(&LuaConfigFile, Option<&str>) -> Result<R, ConfigError>,
    pub(super) names: fn(&R) -> BTreeSet<String>,
}

impl<R> EditedFile<'_, R> {
    pub(in crate::lua_config) fn edit(
        &self,
        name: &str,
        by_hand: String,
        change: Change<'_, R>,
    ) -> Result<Option<String>, ConfigError> {
        let section = Section {
            file: self.file,
            table: self.table,
            entry: name,
            by_hand,
        };
        replace(&self.path, self.holds, |current| {
            let mut names = (self.names)(&(self.read)(self.config, current)?);
            let text = current.unwrap_or(self.missing);
            let edited = match &change {
                Change::Add { .. } if names.contains(name) => Err(section.held_already()),
                Change::Add { entry, .. } => section.add(text, entry),
                Change::Remove => section.remove(text, names.contains(name)),
            }?;
            match &change {
                Change::Add { .. } => names.insert(name.to_string()),
                Change::Remove => names.remove(name),
            };
            self.check(
                &section,
                &change,
                &names,
                &(self.read)(self.config, Some(&edited))?,
            )?;
            Ok(edited)
        })
    }

    fn check(
        &self,
        section: &Section,
        change: &Change<'_, R>,
        expected: &BTreeSet<String>,
        after: &R,
    ) -> Result<(), ConfigError> {
        let names = (self.names)(after);
        let (table, name) = (self.table, section.entry);
        let wrong = match change {
            Change::Add { as_written, .. } if !as_written(after) => {
                format!("would not hold {table} {name}")
            }
            Change::Remove if names.contains(name) => format!("would still hold {table} {name}"),
            _ if &names != expected => format!("would not keep every other {table}"),
            _ => return Ok(()),
        };
        Err(ConfigError::Invalid(format!(
            "the new {} {wrong}",
            self.file
        )))
    }
}
