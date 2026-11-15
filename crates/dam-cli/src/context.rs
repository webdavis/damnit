use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;

use dam_adapters::{Interrupt, OsRandom, ProcessLauncher, SqliteStore, SystemClock};
use dam_application::{
    Access, Clock, Config, ConfigFile, CredentialSource, Dam, FilterFunctions, HelperLauncher,
    Hooks, Randomness, Remotes, Schedule, Store,
};

use crate::caller::{authenticated, caller};
use crate::editor::{EditorSession, EnvEditor};
use crate::error::CliError;
use crate::hook_warnings::HookWarningsOnStandardError;
use crate::output::Format;
use crate::prompt::{Prompt, RefusingPrompt, TerminalPrompt};

pub(crate) struct Context {
    pub(crate) store: Box<dyn Store>,
    pub(crate) config: Config,
    pub(crate) access: Box<dyn Access>,
    pub(crate) config_file: Box<dyn ConfigFile>,
    pub(crate) clock: Box<dyn Clock>,
    pub(crate) random: Box<dyn Randomness>,
    pub(crate) launcher: Box<dyn HelperLauncher>,
    pub(crate) credentials: Box<dyn CredentialSource>,
    pub(crate) filter_functions: Box<dyn FilterFunctions>,
    pub(crate) hooks: Box<dyn Hooks>,
    pub(crate) editor: Option<Box<dyn EditorSession>>,
    pub(crate) prompt: Box<dyn Prompt>,
    pub(crate) schedule: Box<dyn Schedule>,
    pub(crate) tz: jiff::tz::TimeZone,
}

impl Context {
    pub(crate) fn open(
        config_file: Box<dyn ConfigFile>,
        store_path: &Path,
        presented_token: Option<OsString>,
        meta: &BTreeMap<String, String>,
        format: Format,
        interrupt: &Interrupt,
    ) -> Result<Context, CliError> {
        let loaded = config_file.load()?;
        let config = loaded.config;
        let now = SystemClock.now();
        let holder = authenticated(&config, presented_token, now, meta)?;
        let store = SqliteStore::open(store_path)?;
        let holder = holder.map(|h| h.knowing_what_it_made(&store)).transpose()?;
        let access = caller(holder, now, meta);
        let (prompt, editor) =
            terminal_pair_for_a_human_and_refusals_for_a_machine(format, interrupt);
        Ok(Context {
            store: Box::new(store),
            config,
            access,
            config_file,
            clock: Box::new(SystemClock),
            random: Box::new(OsRandom),
            launcher: Box::new(ProcessLauncher::from_env(interrupt.clone())),
            credentials: loaded.credentials,
            filter_functions: loaded.filter_functions,
            hooks: loaded.hooks,
            editor,
            prompt,
            schedule: dam_adapters::this_systems_schedule(),
            tz: jiff::tz::TimeZone::system(),
        })
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
            hook_warnings: &HookWarningsOnStandardError,
        }
    }

    pub(crate) fn remotes(&self) -> Remotes<'_> {
        Remotes {
            launcher: self.launcher.as_ref(),
            credentials: self.credentials.as_ref(),
        }
    }
}

pub(crate) fn terminal_pair_for_a_human_and_refusals_for_a_machine(
    format: Format,
    interrupt: &Interrupt,
) -> (Box<dyn Prompt>, Option<Box<dyn EditorSession>>) {
    match format {
        Format::Human => (
            Box::new(TerminalPrompt::new(interrupt.clone())),
            Some(Box::new(EnvEditor::from_env())),
        ),
        Format::Json | Format::Toon => (Box::new(RefusingPrompt), None),
    }
}
