use std::path::{Path, PathBuf};

use dam_adapters::{
    EnvEditor, OsRandom, ProcessCredentialSource, ProcessLauncher, SqliteStore, SystemClock,
    load_config,
};
use dam_application::{
    Clock, Config, CredentialSource, EditorSession, HelperLauncher, ObjectStore, Randomness,
};

use crate::error::CliError;
use crate::prompt::{Prompt, TerminalPrompt};

pub struct Context {
    pub store: Box<dyn ObjectStore>,
    pub config: Config,
    /// Read by the remote verbs Task 31 adds (`remote add` rewrites the config file at this path).
    #[allow(dead_code)]
    pub config_path: PathBuf,
    pub clock: Box<dyn Clock>,
    pub random: Box<dyn Randomness>,
    /// Read by the push/pull verbs Task 31 adds.
    #[allow(dead_code)]
    pub launcher: Box<dyn HelperLauncher>,
    /// Read by the push/pull verbs Task 31 adds.
    #[allow(dead_code)]
    pub credentials: Box<dyn CredentialSource>,
    pub editor: Box<dyn EditorSession>,
    pub prompt: Box<dyn Prompt>,
    pub tz: jiff::tz::TimeZone,
}

impl Context {
    pub fn open(config_path: PathBuf, store_path: &Path) -> Result<Context, CliError> {
        let config = load_config(&config_path)?;
        let store = SqliteStore::open(store_path)?;
        Ok(Context {
            store: Box::new(store),
            config,
            config_path,
            clock: Box::new(SystemClock),
            random: Box::new(OsRandom),
            launcher: Box::new(ProcessLauncher::from_env()),
            credentials: Box::new(ProcessCredentialSource::from_env()),
            editor: Box::new(EnvEditor::from_env()),
            prompt: Box::new(TerminalPrompt),
            tz: jiff::tz::TimeZone::system(),
        })
    }
}
