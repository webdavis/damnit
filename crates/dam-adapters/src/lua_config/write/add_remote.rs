use dam_application::{Config, ConfigError, RemoteName};

use super::edit::Change;
use super::remote_entry::remote_entry;
use crate::lua_config::LuaConfigFile;

impl LuaConfigFile {
    pub fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError> {
        let name = name.0.as_str();
        let entry = remote_entry(name, url);
        let by_hand = format!("Add the remote to its remote table by hand: {entry}");
        let change = Change::Add {
            entry: &|_| entry.clone(),
            as_written: &|config: &Config| {
                config.remote(name).is_some_and(|remote| remote.url == url)
            },
        };
        self.settings_file().edit(name, by_hand, change)
    }
}
