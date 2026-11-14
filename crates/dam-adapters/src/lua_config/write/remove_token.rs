use dam_application::ConfigError;

use super::edit::Change;
use crate::lua_config::LuaConfigFile;

impl LuaConfigFile {
    pub fn remove_token(&self, name: &str) -> Result<Option<String>, ConfigError> {
        let by_hand = format!("Remove token {name} from it by hand.");
        self.secrets_file().edit(name, by_hand, Change::Remove)
    }
}
