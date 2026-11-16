use std::cell::OnceCell;
use std::path::Path;

use super::store_reading::{self, Store};
use crate::project_scope::Standing;
use dam_adapters::{LuaConfigFile, OpenError};
use dam_application::{
    BackgroundSync, Config, ConfigError, HelperError, HelperLauncher, Progress, RemoteCapabilities,
    RemoteConfig, RunRecords, Step, waited_on,
};
use jiff::Timestamp;

pub(super) struct Answer {
    pub(super) remote: RemoteConfig,
    pub(super) capabilities: Result<RemoteCapabilities, HelperError>,
}

pub(crate) struct Machine<'a> {
    pub(super) config_file: &'a LuaConfigFile,
    pub(super) store_path: &'a Path,
    launcher: &'a dyn HelperLauncher,
    progress: &'a dyn Progress,
    config: OnceCell<Result<Config, ConfigError>>,
    store: OnceCell<Result<Store, OpenError>>,
    answers: OnceCell<Option<Vec<Answer>>>,
    pub(super) background: Box<dyn BackgroundSync>,
    pub(super) run_records: Box<dyn RunRecords>,
    pub(super) now: Timestamp,
    pub(super) standing: Standing,
}

impl<'a> Machine<'a> {
    pub(crate) fn new(
        config_file: &'a LuaConfigFile,
        store_path: &'a Path,
        launcher: &'a dyn HelperLauncher,
        progress: &'a dyn Progress,
    ) -> Machine<'a> {
        Machine {
            config_file,
            store_path,
            launcher,
            progress,
            config: OnceCell::new(),
            store: OnceCell::new(),
            answers: OnceCell::new(),
            background: dam_adapters::no_background_sync(),
            run_records: dam_adapters::no_run_records(),
            now: Timestamp::now(),
            standing: Standing::Outside,
        }
    }

    pub(crate) fn here(
        config_file: &'a LuaConfigFile,
        store_path: &'a Path,
        launcher: &'a dyn HelperLauncher,
        progress: &'a dyn Progress,
    ) -> Machine<'a> {
        Machine::new(config_file, store_path, launcher, progress).watching(
            dam_adapters::background_sync_for_store(store_path),
            dam_adapters::run_records_for_store(store_path),
        )
    }

    pub(crate) fn watching(
        self,
        background: Box<dyn BackgroundSync>,
        run_records: Box<dyn RunRecords>,
    ) -> Machine<'a> {
        Machine {
            background,
            run_records,
            ..self
        }
    }

    pub(crate) fn within(self, standing: Standing) -> Machine<'a> {
        Machine { standing, ..self }
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
        let step = Step {
            name: "check",
            remote: Some(&remote.name.0),
            doing: &format!("checking {}", remote.name.0),
            did: &format!("checked {}", remote.name.0),
            unit: "",
            total: None,
        };
        let capabilities = waited_on(self.progress, step, || {
            self.launcher
                .launch(remote, &[])
                .and_then(|mut helper| helper.capabilities())
        });
        Answer {
            remote: remote.clone(),
            capabilities,
        }
    }
}

#[cfg(test)]
mod tests;
