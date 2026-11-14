use std::path::PathBuf;

use dam_application::{Config, ConfigError};
use mlua::Lua;

use super::evaluate::evaluate;
use super::settings::read_settings;
use super::strict::StrictMap;

pub struct LuaConfigFile {
    config: PathBuf,
}

impl LuaConfigFile {
    pub fn new(config: PathBuf) -> LuaConfigFile {
        LuaConfigFile { config }
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        let lua = Lua::new();
        let Some(table) = evaluate(&lua, &self.config)? else {
            return Ok(Config::default());
        };
        let root = StrictMap::root(&file_name(&self.config), &table)?;
        read_settings(&root)
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
