use dam_application::{Config, EditedTable, Tokens};

use super::edit::EditedFile;
use super::replace::Holds;
use crate::lua_config::LuaConfigFile;

impl LuaConfigFile {
    pub(in crate::lua_config) fn settings_file(&self) -> EditedFile<'_, Config> {
        EditedFile {
            path: self.config_path().to_path_buf(),
            holds: Holds::Settings,
            table: EditedTable::Remotes,
            missing: "return {\n}\n",
            config: self,
            read: LuaConfigFile::settings_in,
            names: |config| {
                config
                    .remotes
                    .iter()
                    .map(|remote| remote.name.0.clone())
                    .collect()
            },
        }
    }

    pub(in crate::lua_config) fn secrets_file(&self) -> EditedFile<'_, Tokens> {
        EditedFile {
            path: self.secrets_path(),
            holds: Holds::Secrets,
            table: EditedTable::Tokens,
            missing: "return {\n  schema = 1,\n}\n",
            config: self,
            read: LuaConfigFile::tokens_in,
            names: |tokens| tokens.names.iter().cloned().collect(),
        }
    }
}
