use std::path::{Path, PathBuf};

use dam_application::{Config, ConfigError, LoadedConfig, Tokens};
use mlua::Lua;

use super::computed::ConfiguredCredentials;
use super::evaluate::{ErrorText, evaluate, read_text, stopped_at_an_error};
use super::secrets::read_secrets;
use super::settings::read_settings;
use super::strict::StrictMap;
use super::token::read_tokens;

pub struct LuaConfigFile {
    config: PathBuf,
}

impl LuaConfigFile {
    pub fn new(config: PathBuf) -> LuaConfigFile {
        LuaConfigFile { config }
    }

    pub fn ends_in_lua(&self) -> Result<(), ConfigError> {
        match self.config.extension().is_some_and(|e| e == "lua") {
            true => Ok(()),
            false => Err(ConfigError::NotLua(self.config.clone())),
        }
    }

    pub fn check(&self) -> Result<Config, ConfigError> {
        self.ends_in_lua()?;
        self.read_both(ErrorText::QuotesWhatItsCodeRaised)
            .map(|loaded| loaded.config)
    }

    pub(super) fn read_both(&self, secrets_error: ErrorText) -> Result<LoadedConfig, ConfigError> {
        let mut config = self.settings_in(read_text(&self.config)?.as_deref())?;
        let lua = Lua::new();
        let path = self.secrets_path();
        let text = read_text(&path)?;
        let secrets = evaluate(&lua, &path, text.as_deref(), secrets_error)?;
        if let Some(table) = &secrets {
            StrictMap::root("secrets.lua", table)
                .and_then(|root| read_secrets(&root))
                .and_then(|read| read.complete(&mut config, &file_name(&self.config)))
                .map_err(|error| secrets_error.about_what_it_holds(&path, error))?;
        }
        let credentials = ConfiguredCredentials::keep(lua, path.clone(), secrets)
            .map_err(|_| stopped_at_an_error(&path))?;
        Ok(LoadedConfig {
            config,
            credentials: Box::new(credentials),
        })
    }

    pub fn config_path(&self) -> &Path {
        &self.config
    }

    pub fn secrets_path(&self) -> PathBuf {
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
