use std::cell::OnceCell;

use dam_adapters::LuaConfigFile;
use dam_application::{Config, ConfigError};

pub(crate) struct Machine<'a> {
    pub(super) config_file: &'a LuaConfigFile,
    config: OnceCell<Result<Config, ConfigError>>,
}

impl<'a> Machine<'a> {
    pub(crate) fn new(config_file: &'a LuaConfigFile) -> Machine<'a> {
        Machine {
            config_file,
            config: OnceCell::new(),
        }
    }

    pub(super) fn config(&self) -> &Result<Config, ConfigError> {
        self.config.get_or_init(|| self.config_file.check())
    }
}
