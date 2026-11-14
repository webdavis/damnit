use std::fmt;

use crate::config::Config;
use crate::ports::{CredentialSource, RemoteName};
use crate::tokens::NewToken;

pub struct LoadedConfig {
    pub config: Config,
    pub credentials: Box<dyn CredentialSource>,
}

pub trait ConfigFile {
    fn load(&self) -> Result<LoadedConfig, ConfigError>;
    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError>;
    fn add_token(&self, token: &NewToken) -> Result<Option<String>, ConfigError>;
    fn remove_token(&self, name: &str) -> Result<Option<String>, ConfigError>;
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Unloadable(String),
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(s) => write!(f, "reading config: {s}"),
            ConfigError::Unloadable(s) | ConfigError::Invalid(s) => write!(f, "config: {s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<String> for ConfigError {
    fn from(refusal: String) -> ConfigError {
        ConfigError::Invalid(refusal)
    }
}
