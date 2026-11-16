use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::IsTerminal;
use std::path::Path;

use dam_adapters::{Interrupt, LuaConfigFile, SqliteStore, SystemClock};
use dam_application::{
    Access, Clock, Config, ConfigFile, CredentialSource, Dam, FilterFunctions, HelperLauncher,
    Hooks, LoadedConfig, Randomness, Remotes, SettingsFile, Store,
};

use crate::caller::{authenticated, caller};
use crate::editor::{EditorSession, EnvEditor};
use crate::error::CliError;
pub(crate) use crate::hook_warnings::HookWarningsOnStandardError;
use crate::output::Format;
use crate::progress::{Asked, ProgressOnStandardError, shown};
use crate::prompt::{Prompt, RefusingPrompt, TerminalPrompt};

pub(crate) struct Context {
    pub(crate) store: Box<dyn Store>,
    pub(crate) config: Config,
    pub(crate) access: Box<dyn Access>,
    pub(crate) config_file: Box<dyn ConfigFile>,
    pub(crate) settings_file: Box<dyn SettingsFile>,
    pub(crate) clock: Box<dyn Clock>,
    pub(crate) random: Box<dyn Randomness>,
    pub(crate) launcher: Box<dyn HelperLauncher>,
    pub(crate) credentials: Box<dyn CredentialSource>,
    pub(crate) filter_functions: Box<dyn FilterFunctions>,
    pub(crate) hooks: Box<dyn Hooks>,
    pub(crate) editor: Option<Box<dyn EditorSession>>,
    pub(crate) prompt: Box<dyn Prompt>,
    pub(crate) tz: jiff::tz::TimeZone,
    pub(crate) changed_by_hook: ChangedByHook,
    pub(crate) interrupt: Interrupt,
    pub(crate) progress: ProgressOnStandardError,
    pub(crate) hook_warnings: HookWarningsOnStandardError,
}

impl Context {
    pub(crate) fn open(
        config_file: LuaConfigFile,
        store_path: &Path,
        presented_token: Option<OsString>,
        meta: &BTreeMap<String, String>,
        format: Format,
        asked: Asked,
        interrupt: &Interrupt,
    ) -> Result<Context, CliError> {
        let loaded = config_file.load()?;
        let config = loaded.config;
        let now = SystemClock.now();
        let holder = authenticated(&config, presented_token, now, meta)?;
        let progress = progress_for(&config, asked, format, std::io::stderr().is_terminal());
        let store = SqliteStore::open_reporting(store_path, &progress)?;
        let holder = holder.map(|h| h.knowing_what_it_made(&store)).transpose()?;
        let access = caller(holder, now, meta);
        let loaded = LoadedConfig { config, ..loaded };
        let opened = reading::Opened {
            config_file: Box::new(config_file.clone()),
            settings_file: Box::new(config_file),
            loaded,
            store: Box::new(store),
            access,
            progress,
        };
        Ok(Context::assembled(opened, format, interrupt))
    }

    pub(crate) fn dam(&self) -> Dam<'_> {
        Dam {
            store: self.store.as_ref(),
            clock: self.clock.as_ref(),
            random: self.random.as_ref(),
            config: &self.config,
            access: self.access.as_ref(),
            filter_functions: self.filter_functions.as_ref(),
            hooks: self.hooks.as_ref(),
            hook_warnings: &self.hook_warnings,
            progress: &self.progress,
            edit_requests: self,
            hook_edits: &self.changed_by_hook,
            interruption: &self.interrupt,
        }
    }

    pub(crate) fn remotes(&self) -> Remotes<'_> {
        Remotes {
            launcher: self.launcher.as_ref(),
            credentials: self.credentials.as_ref(),
        }
    }
}

pub(crate) fn progress_without_a_context(
    config_file: &dyn ConfigFile,
    asked: Asked,
    format: Format,
) -> ProgressOnStandardError {
    let config = config_file.load().map(|loaded| loaded.config);
    let terminal = std::io::stderr().is_terminal();
    progress_for(&config.unwrap_or_default(), asked, format, terminal)
}

pub(crate) fn progress_for(
    config: &Config,
    asked: Asked,
    format: Format,
    terminal: bool,
) -> ProgressOnStandardError {
    let asked = Asked {
        setting: config.progress(),
        ..asked
    };
    ProgressOnStandardError::new(shown(asked, terminal), format)
}

pub(crate) fn terminal_pair_for_a_human_and_refusals_for_a_machine(
    format: Format,
    interrupt: &Interrupt,
) -> (Box<dyn Prompt>, Option<Box<dyn EditorSession>>) {
    match format {
        Format::Human | Format::Quiet => (
            Box::new(TerminalPrompt::new(interrupt.clone())),
            Some(Box::new(EnvEditor::from_env())),
        ),
        Format::Json | Format::Toon => (Box::new(RefusingPrompt), None),
    }
}

mod changed_by_hook;
mod edit_requests;
mod reading;
#[cfg(test)]
mod tests;

pub(crate) use changed_by_hook::ChangedByHook;
