use std::cell::RefCell;

use crate::ports::{ConfigError, SettingChange, SettingsFile};
use crate::setting::{SettingKey, SettingValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Written {
    Set(String, SettingValue),
    Unset(String),
}

#[derive(Default)]
pub struct FakeSettingsFile {
    pub written: RefCell<Vec<Written>>,
    pub refusing: bool,
}

impl FakeSettingsFile {
    pub fn refusing() -> FakeSettingsFile {
        FakeSettingsFile {
            refusing: true,
            ..FakeSettingsFile::default()
        }
    }

    fn write(
        &self,
        key: &SettingKey,
        value: Option<&SettingValue>,
    ) -> Result<SettingChange, ConfigError> {
        let key = key.dotted();
        if self.refusing {
            return Err(ConfigError::SettingReadOnly {
                by_hand: format!("{key} = ..."),
                key,
            });
        }
        let old = self
            .written
            .borrow()
            .iter()
            .rev()
            .find_map(|written| match written {
                Written::Set(name, value) if *name == key => Some(Some(value.text())),
                Written::Unset(name) if *name == key => Some(None),
                _ => None,
            });
        self.written.borrow_mut().push(match value {
            Some(value) => Written::Set(key.clone(), value.clone()),
            None => Written::Unset(key.clone()),
        });
        Ok(SettingChange {
            key,
            old: old.flatten(),
            new: value.map(SettingValue::text),
        })
    }
}

impl SettingsFile for FakeSettingsFile {
    fn set(&self, key: &SettingKey, value: &SettingValue) -> Result<SettingChange, ConfigError> {
        self.write(key, Some(value))
    }

    fn unset(&self, key: &SettingKey) -> Result<SettingChange, ConfigError> {
        self.write(key, None)
    }
}
