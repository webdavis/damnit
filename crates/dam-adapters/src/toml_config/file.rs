use std::path::PathBuf;

use dam_application::{Config, ConfigError, ConfigFile, RemoteName};

use super::{append_remote, load_config};

pub struct TomlConfigFile {
    path: PathBuf,
}

impl TomlConfigFile {
    pub fn new(path: PathBuf) -> TomlConfigFile {
        TomlConfigFile { path }
    }
}

impl ConfigFile for TomlConfigFile {
    fn load(&self) -> Result<Config, ConfigError> {
        load_config(&self.path)
    }

    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError> {
        append_remote(&self.path, &name.0, url)
    }
}

#[cfg(test)]
mod tests;
