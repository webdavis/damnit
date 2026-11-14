use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use dam_application::{ByHand, ConfigError, EditFault, EditedTable, SecretsTightened};

use super::insert::Indent;
use super::replace::{Holds, replace};
use super::section::Section;
use crate::lua_config::LuaConfigFile;
use crate::lua_config::evaluate::read_text;

const OWNER_WRITES: u32 = 0o200;

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
    pub(super) table: EditedTable,
    pub(super) missing: &'static str,
    pub(super) config: &'a LuaConfigFile,
    pub(super) read: fn(&LuaConfigFile, Option<&str>) -> Result<R, ConfigError>,
    pub(super) names: fn(&R) -> BTreeSet<String>,
}

impl<R> EditedFile<'_, R> {
    pub(in crate::lua_config) fn edit(
        &self,
        name: &str,
        by_hand: ByHand,
        change: Change<'_, R>,
    ) -> Result<Option<SecretsTightened>, ConfigError> {
        self.writable(&by_hand)?;
        replace(&self.path, self.holds, |current| {
            self.edited(name, by_hand, &change, current)
        })
    }

    pub(in crate::lua_config) fn can_edit(
        &self,
        name: &str,
        by_hand: ByHand,
        change: Change<'_, R>,
    ) -> Result<(), ConfigError> {
        self.writable(&by_hand)?;
        let current = read_text(&self.path)?;
        self.edited(name, by_hand, &change, current.as_deref())
            .map(|_| ())
    }

    fn writable(&self, by_hand: &ByHand) -> Result<(), ConfigError> {
        match std::fs::metadata(&self.path) {
            Ok(meta) if meta.permissions().mode() & OWNER_WRITES == 0 => {
                Err(ConfigError::ReadOnly {
                    table: self.table,
                    by_hand: by_hand.clone(),
                })
            }
            _ => Ok(()),
        }
    }

    fn edited(
        &self,
        name: &str,
        by_hand: ByHand,
        change: &Change<'_, R>,
        current: Option<&str>,
    ) -> Result<String, ConfigError> {
        let section = Section {
            table: self.table,
            entry: name,
            by_hand,
        };
        let mut names = (self.names)(&(self.read)(self.config, current)?);
        let text = current.unwrap_or(self.missing);
        let edited = match change {
            Change::Add { .. } if names.contains(name) => Err(section.held_already()),
            Change::Add { entry, .. } => section.add(text, entry),
            Change::Remove => section.remove(text, names.contains(name)),
        }?;
        match change {
            Change::Add { .. } => names.insert(name.to_string()),
            Change::Remove => names.remove(name),
        };
        self.check(
            &section,
            change,
            &names,
            &(self.read)(self.config, Some(&edited))?,
        )?;
        Ok(edited)
    }

    fn check(
        &self,
        section: &Section,
        change: &Change<'_, R>,
        expected: &BTreeSet<String>,
        after: &R,
    ) -> Result<(), ConfigError> {
        let names = (self.names)(after);
        let fault = match change {
            Change::Add { as_written, .. } if !as_written(after) => EditFault::WouldNotHoldTheEntry,
            Change::Remove if names.contains(section.entry) => EditFault::WouldStillHoldTheEntry,
            _ if &names != expected => EditFault::WouldNotKeepTheOthers,
            _ => return Ok(()),
        };
        Err(ConfigError::EditWouldBreak {
            table: self.table,
            entry: section.entry.to_string(),
            fault,
        })
    }
}
