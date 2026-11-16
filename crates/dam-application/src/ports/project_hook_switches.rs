use std::path::Path;

use super::config_file::ConfigError;

pub trait ProjectHookSwitches {
    fn switch(&self, project: &Path) -> Result<Option<bool>, ConfigError>;
    fn turn(&self, project: &Path, on: Option<bool>) -> Result<(), ConfigError>;
}
