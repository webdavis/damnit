use std::path::{Path, PathBuf};

use dam_application::{Config, ConfigError, Tokens};
use mlua::Lua;

use super::computed::ConfiguredCredentials;
use super::evaluate::{ErrorText, evaluate, lua_message, read_text};
use super::secrets::read_secrets;
use super::settings::read_settings;
use super::strict::StrictMap;
use super::token::read_tokens;

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

    pub fn load_with_credentials(&self) -> Result<LoadedConfig, ConfigError> {
        let mut config = self.settings_in(read_text(&self.config)?.as_deref())?;
        let lua = Lua::new();
        let path = self.secrets_path();
        let text = read_text(&path)?;
        let secrets = evaluate(&lua, &path, text.as_deref(), ErrorText::NeverQuotesTheFile)?;
        if let Some(table) = &secrets {
            read_secrets(&StrictMap::root("secrets.lua", table)?)?
                .complete(&mut config, &file_name(&self.config))?;
        }
        let credentials = ConfiguredCredentials::keep(lua, path, secrets)
            .map_err(|e| ConfigError::Invalid(lua_message(&e)))?;
        Ok(LoadedConfig {
            config,
            credentials,
        })
    }

    pub(super) fn config_path(&self) -> &Path {
        &self.config
    }

    pub(super) fn secrets_path(&self) -> PathBuf {
        self.config.with_file_name("secrets.lua")
    }

    pub(super) fn settings_in(&self, text: Option<&str>) -> Result<Config, ConfigError> {
        let lua = Lua::new();
        match evaluate(&lua, &self.config, text, ErrorText::QuotesTheFile)? {
            Some(table) => read_settings(&StrictMap::root(&file_name(&self.config), &table)?),
            None => Ok(Config::default()),
        }
    }

    pub(super) fn tokens_in(&self, text: Option<&str>) -> Result<Tokens, ConfigError> {
        let lua = Lua::new();
        let path = self.secrets_path();
        match evaluate(&lua, &path, text, ErrorText::NeverQuotesTheFile)? {
            Some(table) => Ok(read_tokens(&StrictMap::root("secrets.lua", &table)?)),
            None => Ok(Tokens::default()),
        }
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
