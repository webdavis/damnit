use std::path::{Path, PathBuf};

use dam_application::{Config, ConfigError};
use mlua::Lua;

use super::computed::ConfiguredCredentials;
use super::evaluate::{ErrorText, evaluate, lua_message};
use super::secrets::read_secrets;
use super::settings::read_settings;
use super::strict::StrictMap;

pub struct LuaConfigFile {
    config: PathBuf,
}

pub struct LoadedConfig {
    pub config: Config,
    pub credentials: ConfiguredCredentials,
}

impl LuaConfigFile {
    pub fn new(config: PathBuf) -> LuaConfigFile {
        LuaConfigFile { config }
    }

    pub fn load(&self) -> Result<LoadedConfig, ConfigError> {
        let config_name = file_name(&self.config);
        let mut config = match evaluate(&Lua::new(), &self.config, ErrorText::QuotesTheFile)? {
            Some(table) => read_settings(&StrictMap::root(&config_name, &table)?)?,
            None => Config::default(),
        };
        let lua = Lua::new();
        let path = self.config.with_file_name("secrets.lua");
        let secrets = evaluate(&lua, &path, ErrorText::NeverQuotesTheFile)?;
        if let Some(table) = &secrets {
            read_secrets(&StrictMap::root("secrets.lua", table)?)?
                .complete(&mut config, &config_name)?;
        }
        let credentials = ConfiguredCredentials::keep(lua, path, secrets)
            .map_err(|e| ConfigError::Invalid(lua_message(&e)))?;
        Ok(LoadedConfig {
            config,
            credentials,
        })
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
