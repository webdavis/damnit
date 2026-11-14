use dam_application::Config;

use super::edit::EditedFile;
use crate::lua_config::LuaConfigFile;

impl LuaConfigFile {
    pub(in crate::lua_config) fn settings_file(&self) -> EditedFile<'_, Config> {
        EditedFile {
            path: self.config_path().to_path_buf(),
            file: "config.lua",
            table: "remote",
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
}
