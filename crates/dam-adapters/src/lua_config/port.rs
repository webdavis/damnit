use dam_application::{
    Config, ConfigError, ConfigFile, LoadedConfig, NewToken, RemoteName, Tokens,
};

use super::file::LuaConfigFile;
use super::write::{Change, remote_entry, token_entry};

impl ConfigFile for LuaConfigFile {
    fn load(&self) -> Result<LoadedConfig, ConfigError> {
        self.read_both()
    }

    fn add_remote(&self, name: &RemoteName, url: &str) -> Result<Option<String>, ConfigError> {
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

    fn add_token(&self, token: &NewToken) -> Result<Option<String>, ConfigError> {
        let name = token.name.as_str();
        let by_hand = format!(
            "Write the table out in full and run the command again, \
             or add token {name} to its token table by hand."
        );
        let change = Change::Add {
            entry: &|indent| token_entry(token, indent),
            as_written: &|tokens: &Tokens| {
                tokens
                    .entries
                    .iter()
                    .any(|held| held.name == name && held.value == token.value)
            },
        };
        self.secrets_file().edit(name, by_hand, change)
    }

    fn remove_token(&self, name: &str) -> Result<Option<String>, ConfigError> {
        let by_hand = format!("Remove token {name} from it by hand.");
        self.secrets_file().edit(name, by_hand, Change::Remove)
    }
}
