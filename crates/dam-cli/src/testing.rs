use std::cell::RefCell;

use dam_adapters::Interrupt;
use dam_application::testing::{FixedClock, FixedRandom, MemoryStore};
use dam_application::{
    Config, CredentialError, CredentialSource, CredentialSpec, HelperError, HelperLauncher,
    MutationOutcome, MutationVerdict, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    RemoteMutation, Secret,
};
use dam_domain::{Categories, Field, Kind};
use jiff::civil::date;

use crate::context::{Context, terminal_pair_for_a_human_and_refusals_for_a_machine};
use crate::editor::{EditorError, EditorSession};
use crate::error::CliError;
use crate::output::Format;
use crate::prompt::Prompt;

mod config_file;
pub(crate) use config_file::FakeConfigFile;

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

pub(crate) type Save = Box<dyn Fn(&str) -> String>;

pub(crate) struct ScriptedEditor(RefCell<Vec<Save>>, tempfile::TempDir);
impl ScriptedEditor {
    pub(crate) fn saving(saves: Vec<Save>) -> ScriptedEditor {
        ScriptedEditor(RefCell::new(saves), tempfile::tempdir().unwrap())
    }
    pub(crate) fn saving_unchanged() -> ScriptedEditor {
        ScriptedEditor::saving(vec![])
    }
}
impl EditorSession for ScriptedEditor {
    fn edit(&self, opened: &str) -> Result<String, EditorError> {
        let mut saves = self.0.borrow_mut();
        Ok(if saves.is_empty() {
            opened.to_string()
        } else {
            saves.remove(0)(opened)
        })
    }
    fn keep(&self, saved: &str) -> Result<std::path::PathBuf, EditorError> {
        let path = self.1.path().join("kept.toml");
        std::fs::write(&path, saved).map_err(|e| EditorError(e.to_string()))?;
        Ok(path)
    }
}

pub(crate) struct MissingCredentials;
impl CredentialSource for MissingCredentials {
    fn resolve(&self, spec: &CredentialSpec) -> Result<Secret, CredentialError> {
        Err(CredentialError::Missing(spec.name().to_string()))
    }
}

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
                verdict: MutationVerdict::Accepted {
                    remote_id: Some(format!("r-{}", &m.oid.as_str()[..4])),
                },
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
        store: Box::new(MemoryStore::new()),
        config: Config {
            done_interactive: false,
            remotes: vec![],
            categories: Categories::new(vec![]).unwrap_or_else(|_| unreachable!()),
            filters: vec![],
        },
        config_file: Box::new(FakeConfigFile::default()),
        clock: Box::new(FixedClock(date(2026, 9, 18))),
        random: Box::new(FixedRandom::new(1)),
        launcher: Box::new(EchoLauncher {
            pulled: RefCell::new(None),
        }),
        credentials: Box::new(MissingCredentials),
        editor: Some(Box::new(ScriptedEditor::saving_unchanged())),
        prompt: Box::new(ScriptedPrompt {
            choices: RefCell::new(vec![]),
            texts: RefCell::new(vec![]),
        }),
        tz: jiff::tz::TimeZone::UTC,
        no_pull: false,
    }
}

pub(crate) fn machine_context() -> Context {
    let (prompt, editor) =
        terminal_pair_for_a_human_and_refusals_for_a_machine(Format::Json, &Interrupt::default());
    Context {
        prompt,
        editor,
        ..context()
    }
}
