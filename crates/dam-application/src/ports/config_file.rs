use std::fmt;

use crate::config::Config;
use crate::ports::RemoteName;

pub trait ConfigFile {
    fn load(&self) -> Result<Config, ConfigError>;
    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError>;
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Syntax(String),
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(s) => write!(f, "reading config: {s}"),
            ConfigError::Syntax(s) => write!(f, "config is not TOML: {s}"),
            ConfigError::Invalid(s) => write!(f, "config: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<String> for ConfigError {
    fn from(refusal: String) -> ConfigError {
        ConfigError::Invalid(refusal)
    }
}
