use std::path::{Path, PathBuf};

use dam_application::{Config, ConfigError};
use mlua::Lua;

use super::evaluate::{ErrorText, evaluate};
use super::secrets::read_secrets;
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
        let config_name = file_name(&self.config);
        let mut config = match evaluate(&Lua::new(), &self.config, ErrorText::QuotesTheFile)? {
            Some(table) => read_settings(&StrictMap::root(&config_name, &table)?)?,
            None => Config::default(),
        };
        let lua = Lua::new();
        let secrets = self.config.with_file_name("secrets.lua");
        if let Some(table) = evaluate(&lua, &secrets, ErrorText::NeverQuotesTheFile)? {
            read_secrets(&StrictMap::root("secrets.lua", &table)?)?
                .complete(&mut config, &config_name)?;
        }
        Ok(config)
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
