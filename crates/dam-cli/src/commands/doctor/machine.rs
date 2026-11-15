use std::cell::OnceCell;
use std::path::Path;

use super::store_reading::{self, Store};
use dam_adapters::{LuaConfigFile, OpenError};
use dam_application::{
    Config, ConfigError, HelperError, HelperLauncher, RemoteCapabilities, RemoteConfig,
};

pub(super) struct Answer {
    pub(super) remote: RemoteConfig,
    pub(super) capabilities: Result<RemoteCapabilities, HelperError>,
}

pub(crate) struct Machine<'a> {
    pub(super) config_file: &'a LuaConfigFile,
    pub(super) store_path: &'a Path,
    launcher: &'a dyn HelperLauncher,
    config: OnceCell<Result<Config, ConfigError>>,
    store: OnceCell<Result<Store, OpenError>>,
    answers: OnceCell<Option<Vec<Answer>>>,
}

impl<'a> Machine<'a> {
    pub(crate) fn new(
        config_file: &'a LuaConfigFile,
        store_path: &'a Path,
        launcher: &'a dyn HelperLauncher,
    ) -> Machine<'a> {
        Machine {
            config_file,
            store_path,
            launcher,
            config: OnceCell::new(),
            store: OnceCell::new(),
            answers: OnceCell::new(),
        }
    }

    pub(super) fn config(&self) -> &Result<Config, ConfigError> {
        self.config.get_or_init(|| self.config_file.check())
    }

    pub(super) fn store(&self) -> &Result<Store, OpenError> {
        self.store.get_or_init(|| {
            let remotes = self.config().as_ref().map_or(&[][..], |c| &c.remotes);
            store_reading::read(self.store_path, remotes)
        })
    }

    pub(super) fn answers(&self) -> Option<&[Answer]> {
        let answers = self.answers.get_or_init(|| {
            // The store is read and closed before a helper starts, since a helper may take
            // its whole deadline while another dam writes to the store.
            let _ = self.store();
            let remotes = &self.config().as_ref().ok()?.remotes;
            Some(remotes.iter().map(|remote| self.ask(remote)).collect())
        });
        answers.as_deref()
    }

    fn ask(&self, remote: &RemoteConfig) -> Answer {
        let capabilities = self
            .launcher
            .launch(remote, &[])
            .and_then(|mut helper| helper.capabilities());
        Answer {
            remote: remote.clone(),
            capabilities,
        }
    }
}

#[cfg(test)]
mod tests;
