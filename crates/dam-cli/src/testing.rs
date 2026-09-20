//! Test doubles for `Context`, consumed by every verb's own test module.

use std::cell::RefCell;

use dam_adapters::SqliteStore;
use dam_application::{
    Clock, Config, CredentialError, CredentialSource, CredentialSpec, EditorError, EditorSession,
    HelperError, HelperLauncher, MutationOutcome, PullOutcome, RemoteCapabilities, RemoteConfig,
    RemoteHelper, RemoteMutation, Secret,
};
use dam_domain::{Categories, Date, Field, Kind, Timestamp};
use jiff::civil::date;

use crate::context::{Context, RefusingEditor};
use crate::error::CliError;
use crate::prompt::{Prompt, RefusingPrompt};

pub(crate) struct FixedClock(pub(crate) Date);
impl Clock for FixedClock {
    fn today(&self) -> Date {
        self.0
    }
    fn now(&self) -> Timestamp {
        self.0
            .at(0, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .map(|z| z.timestamp())
            .unwrap_or(Timestamp::UNIX_EPOCH)
    }
}

pub(crate) struct CountingRandom(pub(crate) u8);
impl dam_application::Randomness for CountingRandom {
    fn fill(&mut self, buf: &mut [u8]) {
        self.0 = self.0.wrapping_add(1);
        buf.fill(self.0);
    }
}

/// Answers every choice with the scripted index and every text with the scripted string.
pub(crate) struct ScriptedPrompt {
    pub(crate) choices: RefCell<Vec<usize>>,
    pub(crate) texts: RefCell<Vec<String>>,
}
impl Prompt for ScriptedPrompt {
    fn choose(&self, _: &str, _: &[&str]) -> Result<usize, CliError> {
        self.choices.borrow_mut().pop().ok_or(CliError::Cancelled)
    }
    fn text(&self, _: &str) -> Result<String, CliError> {
        self.texts.borrow_mut().pop().ok_or(CliError::Cancelled)
    }
}

/// Returns the scripted text as the editor's save.
pub(crate) struct ScriptedEditor(pub(crate) String);
impl EditorSession for ScriptedEditor {
    fn edit(&self, _: &str) -> Result<String, EditorError> {
        Ok(self.0.clone())
    }
}

/// Supplies no credential at all, whatever is asked for.
pub(crate) struct MissingCredentials;
impl CredentialSource for MissingCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        Err(CredentialError::Missing(spec.name().to_string()))
    }
}

/// A helper that accepts every task, pulls the scripted objects once, and reports every push as ok.
#[derive(Debug)]
pub(crate) struct EchoHelper {
    pub(crate) pulled: PullOutcome,
}
impl RemoteHelper for EchoHelper {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError> {
        Ok(RemoteCapabilities {
            kinds: vec![Kind::Task],
            fields: vec![
                Field::Subject,
                Field::Body,
                Field::Path,
                Field::Labels,
                Field::Done,
                Field::Priority,
                Field::Due,
            ],
            credentials: vec![],
            incremental: false,
        })
    }
    fn pull(&mut self, _: Option<&str>) -> Result<PullOutcome, HelperError> {
        Ok(std::mem::take(&mut self.pulled))
    }
    fn push(
        &mut self,
        mutations: Vec<RemoteMutation>,
    ) -> Result<Vec<MutationOutcome>, HelperError> {
        Ok(mutations
            .iter()
            .map(|m| MutationOutcome {
                oid: m.oid.clone(),
                ok: true,
                remote_id: Some(format!("r-{}", &m.oid.as_str()[..4])),
                why: None,
            })
            .collect())
    }
}

#[derive(Debug)]
pub(crate) struct EchoLauncher {
    pub(crate) pulled: RefCell<Option<PullOutcome>>,
}
impl HelperLauncher for EchoLauncher {
    fn launch(
        &self,
        _: &RemoteConfig,
        _: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Ok(Box::new(EchoHelper {
            pulled: self.pulled.borrow_mut().take().unwrap_or_default(),
        }))
    }
}

/// A launcher whose helper is never reachable, for the auto-pull failure path.
#[derive(Debug)]
pub(crate) struct FailingLauncher;
impl HelperLauncher for FailingLauncher {
    fn launch(
        &self,
        remote: &RemoteConfig,
        _: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Err(HelperError::NotFound {
            helper: remote.helper.clone(),
        })
    }
}

pub(crate) fn context() -> Context {
    Context {
        store: Box::new(SqliteStore::in_memory().unwrap_or_else(|e| panic!("{e}"))),
        config: Config {
            done_interactive: false,
            remotes: vec![],
            categories: Categories::new(vec![]).unwrap_or_else(|_| unreachable!()),
            filters: vec![],
        },
        config_path: std::path::PathBuf::from("/nonexistent/config.toml"),
        clock: Box::new(FixedClock(date(2026, 9, 18))),
        random: Box::new(CountingRandom(0)),
        launcher: Box::new(EchoLauncher {
            pulled: RefCell::new(None),
        }),
        credentials: Box::new(MissingCredentials),
        editor: Box::new(ScriptedEditor(String::new())),
        prompt: Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![]),
            texts: RefCell::new(vec![]),
        }),
        tz: jiff::tz::TimeZone::UTC,
    }
}

/// A context wired the way `--json`/`--toon` wires one: any question refuses
/// instead of blocking, matching what `Context::open` installs for those formats.
pub(crate) fn machine_context() -> Context {
    Context {
        prompt: Box::new(RefusingPrompt),
        editor: Box::new(RefusingEditor),
        ..context()
    }
}
