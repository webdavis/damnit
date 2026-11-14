use dam_application::{
    ByHand, Config, ConfigError, ConfigFile, LoadedConfig, NewToken, RemoteName, SecretsTightened,
    TokenName, Tokens,
};

use super::evaluate::ErrorText;
use super::file::LuaConfigFile;
use super::write::{Change, bare_token_entry, remote_entry, token_entry};

impl ConfigFile for LuaConfigFile {
    fn load(&self) -> Result<LoadedConfig, ConfigError> {
        self.read_both(ErrorText::NeverQuotesTheFile)
    }

    fn add_remote(
        &self,
        name: &RemoteName,
        url: &str,
    ) -> Result<Option<SecretsTightened>, ConfigError> {
        let name = name.0.as_str();
        let entry = remote_entry(name, url);
        let by_hand = ByHand::AddRemote {
            entry: entry.clone(),
        };
        let change = Change::Add {
            entry: &|_| entry.clone(),
            as_written: &|config: &Config| {
                config.remote(name).is_some_and(|remote| remote.url == url)
            },
        };
        self.settings_file().edit(name, by_hand, change)
    }

    fn can_add_token(&self, name: &TokenName) -> Result<(), ConfigError> {
        let name = name.as_str();
        let change = Change::Add {
            entry: &|_| bare_token_entry(name),
            as_written: &|tokens: &Tokens| tokens.names.iter().any(|held| held == name),
        };
        self.secrets_file()
            .can_edit(name, adding_token_by_hand(name), change)
    }

    fn add_token(&self, token: &NewToken) -> Result<Option<SecretsTightened>, ConfigError> {
        let name = token.name.as_str();
        let by_hand = adding_token_by_hand(name);
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

    fn remove_token(&self, name: &str) -> Result<Option<SecretsTightened>, ConfigError> {
        let by_hand = ByHand::RemoveToken {
            name: name.to_string(),
        };
        self.secrets_file().edit(name, by_hand, Change::Remove)
    }
}

fn adding_token_by_hand(name: &str) -> ByHand {
    ByHand::AddToken {
        name: name.to_string(),
    }
}
