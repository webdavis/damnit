use std::path::{Path, PathBuf};

use dam_adapters::{
    EnvEditor, OsRandom, ProcessCredentialSource, ProcessLauncher, SqliteStore, SystemClock,
    load_config,
};
use dam_application::{
    Clock, Config, CredentialSource, EditorError, EditorSession, HelperLauncher, Randomness, Store,
};

use crate::error::CliError;
use crate::output::Format;
use crate::prompt::{Prompt, RefusingPrompt, TerminalPrompt};

/// Installed instead of `EnvEditor` for `--json`/`--toon`, so `edit -e` fails
/// instead of opening `$EDITOR`.
pub(crate) struct RefusingEditor;

impl EditorSession for RefusingEditor {
    fn edit(&self, _text: &str) -> Result<String, EditorError> {
        Err(EditorError(
            "-e opens an editor; drop --json/--toon to use it".into(),
        ))
    }
}

pub(crate) struct Context {
    pub(crate) store: Box<dyn Store>,
    pub(crate) config: Config,
    /// `remote add` rewrites the config file at this path.
    pub(crate) config_path: PathBuf,
    pub(crate) clock: Box<dyn Clock>,
    pub(crate) random: Box<dyn Randomness>,
    pub(crate) launcher: Box<dyn HelperLauncher>,
    pub(crate) credentials: Box<dyn CredentialSource>,
    pub(crate) editor: Box<dyn EditorSession>,
    pub(crate) prompt: Box<dyn Prompt>,
    pub(crate) tz: jiff::tz::TimeZone,
}

impl Context {
    /// `format` picks the prompt and editor: `Human` gets the real terminal
    /// pair, `Json`/`Toon` get the refusing pair so a verb that needs to ask
    /// something fails cleanly instead of blocking or opening an editor.
    pub(crate) fn open(
        config_path: PathBuf,
        store_path: &Path,
        format: Format,
    ) -> Result<Context, CliError> {
        let config = load_config(&config_path)?;
        let store = SqliteStore::open(store_path)?;
        let (prompt, editor): (Box<dyn Prompt>, Box<dyn EditorSession>) = match format {
            Format::Human => (
                Box::new(TerminalPrompt::default()),
                Box::new(EnvEditor::from_env()),
            ),
            Format::Json | Format::Toon => (Box::new(RefusingPrompt), Box::new(RefusingEditor)),
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
        })
    }
}
