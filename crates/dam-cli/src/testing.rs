//! Test doubles for `Context`, consumed by the verb tests Tasks 29 to 31 add.
#![allow(dead_code)]

use std::cell::RefCell;

use dam_adapters::SqliteStore;
use dam_application::{
    Capabilities, Clock, Config, CredentialError, CredentialSource, CredentialSpec, EditorError,
    EditorSession, HelperError, HelperLauncher, Mutation, MutationResult, PullResponse,
    PushResponse, RemoteConfig, RemoteHelper,
};
use dam_domain::{Categories, Date, Timestamp};
use jiff::civil::date;

use crate::context::Context;
use crate::error::CliError;
use crate::prompt::Prompt;

pub struct FixedClock(pub Date);
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

pub struct CountingRandom(pub u8);
impl dam_application::Randomness for CountingRandom {
    fn fill(&mut self, buf: &mut [u8]) {
        self.0 = self.0.wrapping_add(1);
        buf.fill(self.0);
    }
}

/// Answers every choice with the scripted index and every text with the scripted string.
pub struct ScriptedPrompt {
    pub choices: RefCell<Vec<usize>>,
    pub texts: RefCell<Vec<String>>,
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
pub struct ScriptedEditor(pub String);
impl EditorSession for ScriptedEditor {
    fn edit(&self, _: &str) -> Result<String, EditorError> {
        Ok(self.0.clone())
    }
}

pub struct NoCredentials;
impl CredentialSource for NoCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<String, CredentialError> {
        Err(CredentialError::Missing(spec.name().to_string()))
    }
}

/// A helper that accepts every task, pulls the scripted objects once, and reports every push as ok.
#[derive(Debug)]
pub struct EchoHelper {
    pub pulled: PullResponse,
}
impl RemoteHelper for EchoHelper {
    fn capabilities(&mut self) -> Result<Capabilities, HelperError> {
        Ok(Capabilities {
            protocol: 1,
            kinds: vec!["task".into()],
            fields: vec![
                "subject".into(),
                "body".into(),
                "path".into(),
                "labels".into(),
                "done".into(),
                "priority".into(),
                "due".into(),
            ],
            credentials: vec![],
            incremental: false,
        })
    }
    fn pull(&mut self, _: Option<&str>) -> Result<PullResponse, HelperError> {
        Ok(std::mem::replace(
            &mut self.pulled,
            PullResponse {
                objects: vec![],
                removed: vec![],
                sync: None,
            },
        ))
    }
    fn push(&mut self, mutations: Vec<Mutation>) -> Result<PushResponse, HelperError> {
        Ok(PushResponse {
            results: mutations
                .iter()
                .map(|m| MutationResult {
                    oid: m.oid.clone(),
                    ok: true,
                    remote_id: Some(format!("r-{}", &m.oid[..4])),
                    why: None,
                })
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct EchoLauncher {
    pub pulled: RefCell<Option<PullResponse>>,
}
impl HelperLauncher for EchoLauncher {
    fn launch(
        &self,
        _: &RemoteConfig,
        _: &[(String, String)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        Ok(Box::new(EchoHelper {
            pulled: self.pulled.borrow_mut().take().unwrap_or(PullResponse {
                objects: vec![],
                removed: vec![],
                sync: None,
            }),
        }))
    }
}

pub fn context() -> Context {
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
        credentials: Box::new(NoCredentials),
        editor: Box::new(ScriptedEditor(String::new())),
        prompt: Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![]),
            texts: RefCell::new(vec![]),
        }),
        tz: jiff::tz::TimeZone::UTC,
    }
}
