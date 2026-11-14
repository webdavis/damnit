use dam_application::{ConfigError, NewToken, Tokens};

use super::edit::Change;
use super::token_entry::token_entry;
use crate::lua_config::LuaConfigFile;

impl LuaConfigFile {
    pub fn add_token(&self, token: &NewToken) -> Result<Option<String>, ConfigError> {
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
}
