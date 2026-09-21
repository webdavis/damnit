use std::path::{Path, PathBuf};

use dam_adapters::{
    EnvEditor, OsRandom, ProcessCredentialSource, ProcessLauncher, SqliteStore, SystemClock,
    load_config,
};
use dam_application::{
    Clock, Config, CredentialSource, EditorSession, HelperLauncher, Randomness, Store,
};

use crate::error::CliError;
use crate::output::Format;
use crate::prompt::{Prompt, RefusingPrompt, TerminalPrompt};

pub(crate) struct Context {
    pub(crate) store: Box<dyn Store>,
    pub(crate) config: Config,
    /// `remote add` rewrites the config file at this path.
    pub(crate) config_path: PathBuf,
    pub(crate) clock: Box<dyn Clock>,
    pub(crate) random: Box<dyn Randomness>,
    pub(crate) launcher: Box<dyn HelperLauncher>,
    pub(crate) credentials: Box<dyn CredentialSource>,
    /// `None` under `--json`/`--toon`, which cannot open one.
    pub(crate) editor: Option<Box<dyn EditorSession>>,
    pub(crate) prompt: Box<dyn Prompt>,
    pub(crate) tz: jiff::tz::TimeZone,
    /// Set by `--no-pull`: a read answers from the store and pulls nothing.
    pub(crate) no_pull: bool,
}

impl Context {
    /// `format` picks the prompt and editor: `Human` gets the real terminal
    /// pair, `Json`/`Toon` get a refusing prompt and no editor at all, so a
    /// verb needing either refuses instead of blocking or opening one.
    pub(crate) fn open(
        config_path: PathBuf,
        store_path: &Path,
        format: Format,
        no_pull: bool,
    ) -> Result<Context, CliError> {
        let config = load_config(&config_path)?;
        let store = SqliteStore::open(store_path)?;
        let (prompt, editor): (Box<dyn Prompt>, Option<Box<dyn EditorSession>>) = match format {
            Format::Human => (
                Box::new(TerminalPrompt::default()),
                Some(Box::new(EnvEditor::from_env())),
            ),
            Format::Json | Format::Toon => (Box::new(RefusingPrompt), None),
        };
        Ok(Context {
            store: Box::new(store),
            config,
            config_path,
            clock: Box::new(SystemClock),
            random: Box::new(OsRandom),
            launcher: Box::new(ProcessLauncher::from_env()),
            credentials: Box::new(ProcessCredentialSource::from_env()),
            editor,
            prompt,
            tz: jiff::tz::TimeZone::system(),
            no_pull,
        })
    }
}
