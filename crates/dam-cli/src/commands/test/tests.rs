mod credentials;
mod progress;
mod report;

use std::cell::RefCell;
use std::rc::Rc;

use dam_adapters::LuaConfigFile;
use dam_application::testing::{FakeProgress, ScriptedHelper, task_caps};
use dam_application::{
    Declared, HelperError, HelperLauncher, PullOutcome, PushOutcome, RemoteCapabilities,
    RemoteConfig, RemoteHelper, RemoteMutation, Secret,
};
use tempfile::TempDir;

use crate::args::PartName;
use crate::commands::check_report::CheckReport;

const MARKER: &str = "refused-MARKER-5c1e";
const FAKE: &str = "return { remote = { fake = { url = \"fake::\" } } }\n";
const FAKE_TOKEN: &str = "return { schema = 1, remote = { fake = { api_token = \"tok\" } } }\n";

#[derive(Default)]
struct Helpers {
    logins: Rc<RefCell<Vec<String>>>,
}

impl HelperLauncher for Helpers {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        let answers_login = match remote.helper.as_str() {
            "fake" => true,
            "old" => false,
            other => {
                return Err(HelperError::NotFound {
                    helper: format!("dam-remote-{other}"),
                });
            }
        };
        Ok(Box::new(LoginHelper {
            scripted: ScriptedHelper {
                caps: RemoteCapabilities {
                    answers_login,
                    ..task_caps()
                },
                ..ScriptedHelper::answering(Ok(PullOutcome::default()))
            },
            remote: remote.name.0.clone(),
            token: credentials
                .first()
                .map(|(_, value)| value.expose().to_string()),
            logins: self.logins.clone(),
        }))
    }
}

#[derive(Debug)]
struct LoginHelper {
    scripted: ScriptedHelper,
    remote: String,
    token: Option<String>,
    logins: Rc<RefCell<Vec<String>>>,
}

impl RemoteHelper for LoginHelper {
    fn capabilities(&mut self) -> Result<RemoteCapabilities, HelperError> {
        self.scripted.capabilities()
    }
    fn pull(&mut self, since: Option<&str>) -> Result<PullOutcome, HelperError> {
        self.scripted.pull(since)
    }
    fn push(&mut self, mutations: Vec<RemoteMutation>) -> Result<PushOutcome, HelperError> {
        self.scripted.push(mutations)
    }
    fn login(&mut self) -> Result<(), HelperError> {
        self.logins.borrow_mut().push(self.remote.clone());
        match self.token.as_deref() {
            Some("tok") => Ok(()),
            token => Err(HelperError::Remote(format!(
                "fake answered 401: token {} is not valid",
                token.unwrap_or_default()
            ))),
        }
    }
    fn declared_on_the_wire(&self, mutation: &RemoteMutation) -> Result<Declared, HelperError> {
        self.scripted.declared_on_the_wire(mutation)
    }
}

struct Files {
    _dir: TempDir,
    config: LuaConfigFile,
    progress: FakeProgress,
}

impl Files {
    fn new(config_lua: &str, secrets_lua: &str) -> Files {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.lua"), config_lua).unwrap();
        std::fs::write(dir.path().join("secrets.lua"), secrets_lua).unwrap();
        let config = LuaConfigFile::new(dir.path().join("config.lua"));
        Files {
            _dir: dir,
            config,
            progress: FakeProgress::default(),
        }
    }

    fn report(&self, launcher: &Helpers, asked: &[PartName]) -> CheckReport {
        super::report(asked, &self.config, launcher, &self.progress)
    }
}

fn human(report: &CheckReport) -> Vec<String> {
    let human = crate::view::report(report).unwrap().human;
    human.lines().map(str::to_string).collect()
}

fn rows_of(report: &CheckReport) -> Vec<String> {
    human(report)
        .into_iter()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .collect()
}

fn attention_of(report: &CheckReport) -> Vec<String> {
    human(report)
        .into_iter()
        .skip_while(|line| line != "Needs attention")
        .skip(1)
        .collect()
}

fn json(report: &CheckReport) -> serde_json::Value {
    serde_json::to_value(report).unwrap()
}
