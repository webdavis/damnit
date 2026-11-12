use std::path::PathBuf;

use dam_application::{Config, ConfigError, ConfigFile, NewToken, RemoteName};

use super::create::create_if_missing;
use super::token_append::append_token;
use super::{append_remote, load_config};

pub struct TomlConfigFile {
    path: PathBuf,
}

impl TomlConfigFile {
    pub fn new(path: PathBuf) -> TomlConfigFile {
        TomlConfigFile { path }
    }

    pub fn create_if_missing(&self) -> std::io::Result<bool> {
        create_if_missing(&self.path)
    }
}

impl ConfigFile for TomlConfigFile {
    fn load(&self) -> Result<Config, ConfigError> {
        load_config(&self.path)
    }

    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError> {
        append_remote(&self.path, &name.0, url)
    }

    fn add_token(&self, token: &NewToken) -> Result<Option<String>, ConfigError> {
        append_token(&self.path, token)
    }
}

#[cfg(test)]
mod tests;
