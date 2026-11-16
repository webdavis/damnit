use super::config_file::ConfigError;
use crate::setting::{SettingKey, SettingValue};

#[derive(Debug, PartialEq, Eq)]
pub struct SettingChange {
    pub key: String,
    pub old: Option<String>,
    pub new: Option<String>,
}

pub trait SettingsFile {
    fn set(&self, key: &SettingKey, value: &SettingValue) -> Result<SettingChange, ConfigError>;
    fn unset(&self, key: &SettingKey) -> Result<SettingChange, ConfigError>;
}
