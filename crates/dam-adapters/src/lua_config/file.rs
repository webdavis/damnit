use std::path::{Path, PathBuf};
use std::time::Duration;

use dam_application::{Config, ConfigError, LoadedConfig, Tokens};
use mlua::{Lua, Table};

use super::computed::ConfiguredCredentials;
use super::evaluate::{ErrorText, evaluate, read_text, stopped_at_an_error};
use super::functions::{LuaFunctions, limit_every_thread};
use super::hook_folder::{HOOKS_FOLDER, KeptHook, configured, read_hook_folder};
use super::hook_marker::mark_what_a_hook_starts;
use super::secrets::read_secrets;
use super::settings::read_settings;
use super::strict::StrictMap;
use super::token::read_tokens;

const FUNCTION_LIMIT: Duration = Duration::from_secs(5);

pub(super) struct Kept {
    pub(super) config: Config,
    pub(super) lua: Lua,
    pub(super) settings: Option<Table>,
    pub(super) hooks: Vec<KeptHook>,
}

#[derive(Clone)]
pub struct LuaConfigFile {
    config: PathBuf,
    function_limit: Duration,
}

impl LuaConfigFile {
    pub fn new(config: PathBuf) -> LuaConfigFile {
        LuaConfigFile {
            config,
            function_limit: FUNCTION_LIMIT,
        }
    }

    #[cfg(test)]
    pub(super) fn limiting_functions_to(self, function_limit: Duration) -> LuaConfigFile {
        LuaConfigFile {
            function_limit,
            ..self
        }
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
        let Kept {
            mut config,
            lua: settings_lua,
            settings,
            hooks,
        } = self.settings_kept(read_text(&self.config)?.as_deref())?;
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
        let functions = LuaFunctions::keep(
            settings_lua,
            self.config.clone(),
            settings,
            hooks,
            config.owner_name(),
            self.function_limit,
        )
        .map_err(|_| stopped_at_an_error(&self.config))?;
        Ok(LoadedConfig {
            config,
            credentials: Box::new(credentials),
            filter_functions: Box::new(functions.clone()),
            hooks: Box::new(functions),
        })
    }

    pub fn config_path(&self) -> &Path {
        &self.config
    }

    pub fn secrets_path(&self) -> PathBuf {
        self.config.with_file_name("secrets.lua")
    }

    pub(super) fn settings_in(&self, text: Option<&str>) -> Result<Config, ConfigError> {
        self.settings_kept(text).map(|kept| kept.config)
    }

    pub(super) fn settings_kept(&self, text: Option<&str>) -> Result<Kept, ConfigError> {
        let lua = Lua::new();
        limit_every_thread(&lua)
            .and_then(|()| mark_what_a_hook_starts(&lua))
            .map_err(|_| stopped_at_an_error(&self.config))?;
        let table = evaluate(&lua, &self.config, text, ErrorText::QuotesTheFile)?;
        let mut config = match &table {
            Some(table) => read_settings(&StrictMap::root(&file_name(&self.config), table)?)?,
            None => Config::default(),
        };
        let hooks = read_hook_folder(&lua, &self.config.with_file_name(HOOKS_FOLDER))?;
        (config.hooks, config.task_pulled) = configured(&hooks);
        Ok(Kept {
            config,
            lua,
            settings: table,
            hooks,
        })
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
