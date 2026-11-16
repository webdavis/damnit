use std::path::PathBuf;

use dam_application::{ConfigError, HookFile};
use mlua::Lua;

use super::evaluate::stopped_at_an_error;
use super::file::LuaConfigFile;
use super::functions::limit_every_thread;
use super::hook_folder::{HOOKS_FOLDER, HookFolder, list_hook_folder};
use super::hook_marker::mark_what_a_hook_starts;

impl LuaConfigFile {
    pub fn hook_files(&self) -> Result<Vec<HookFile>, ConfigError> {
        let folder = self.hooks_folder();
        Ok(list_hook_folder(
            &self.fenced_lua()?,
            &HookFolder::personal(&folder)?,
        ))
    }

    pub(super) fn hooks_folder(&self) -> PathBuf {
        self.config_path().with_file_name(HOOKS_FOLDER)
    }

    pub(super) fn fenced_lua(&self) -> Result<Lua, ConfigError> {
        let lua = Lua::new();
        limit_every_thread(&lua)
            .and_then(|()| mark_what_a_hook_starts(&lua))
            .map_err(|_| stopped_at_an_error(self.config_path()))?;
        Ok(lua)
    }
}
