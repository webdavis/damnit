use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use dam_adapters::{LuaConfigFile, OpenError, ReadOnly, SqliteStore};
use dam_application::{Config, ConfigError};

pub(super) enum Store {
    NoneYet,
    LeftOver(Vec<PathBuf>),
    Read(Reading),
}

pub(super) struct Reading {
    pub(super) version: u32,
}

pub(crate) struct Machine<'a> {
    pub(super) config_file: &'a LuaConfigFile,
    pub(super) store_path: &'a Path,
    config: OnceCell<Result<Config, ConfigError>>,
    store: OnceCell<Result<Store, OpenError>>,
}

impl<'a> Machine<'a> {
    pub(crate) fn new(config_file: &'a LuaConfigFile, store_path: &'a Path) -> Machine<'a> {
        Machine {
            config_file,
            store_path,
            config: OnceCell::new(),
            store: OnceCell::new(),
        }
    }

    pub(super) fn config(&self) -> &Result<Config, ConfigError> {
        self.config.get_or_init(|| self.config_file.check())
    }

    pub(super) fn store(&self) -> &Result<Store, OpenError> {
        self.store
            .get_or_init(|| match SqliteStore::open_read_only(self.store_path)? {
                ReadOnly::NoStore => Ok(Store::NoneYet),
                ReadOnly::LeftOver(files) => Ok(Store::LeftOver(files)),
                ReadOnly::Opened(store) => Ok(Store::Read(Reading {
                    version: store.schema_version()?,
                })),
            })
    }
}
