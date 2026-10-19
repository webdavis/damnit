use std::path::Path;

use dam_adapters::{
    Interrupt, OsRandom, ProcessCredentialSource, ProcessLauncher, SqliteStore, SystemClock,
};
use dam_application::{
    Clock, Config, ConfigFile, CredentialSource, HelperLauncher, Randomness, Store,
};

use crate::editor::{EditorSession, EnvEditor};
use crate::error::CliError;
use crate::output::Format;
use crate::prompt::{Prompt, RefusingPrompt, TerminalPrompt};

pub(crate) struct Context {
    pub(crate) store: Box<dyn Store>,
    pub(crate) config: Config,
    pub(crate) config_file: Box<dyn ConfigFile>,
    pub(crate) clock: Box<dyn Clock>,
    pub(crate) random: Box<dyn Randomness>,
    pub(crate) launcher: Box<dyn HelperLauncher>,
    pub(crate) credentials: Box<dyn CredentialSource>,
    pub(crate) editor: Option<Box<dyn EditorSession>>,
    pub(crate) prompt: Box<dyn Prompt>,
    pub(crate) tz: jiff::tz::TimeZone,
    pub(crate) no_pull: bool,
}

impl Context {
    pub(crate) fn open(
        config_file: Box<dyn ConfigFile>,
        store_path: &Path,
        format: Format,
        no_pull: bool,
        interrupt: &Interrupt,
    ) -> Result<Context, CliError> {
        let config = config_file.load()?;
        let store = SqliteStore::open(store_path)?;
        let (prompt, editor) =
            terminal_pair_for_a_human_and_refusals_for_a_machine(format, interrupt);
        Ok(Context {
            store: Box::new(store),
            config,
            config_file,
            clock: Box::new(SystemClock),
            random: Box::new(OsRandom),
            launcher: Box::new(ProcessLauncher::from_env(interrupt.clone())),
            credentials: Box::new(ProcessCredentialSource::from_env()),
            editor,
            prompt,
            tz: jiff::tz::TimeZone::system(),
            no_pull,
        })
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
