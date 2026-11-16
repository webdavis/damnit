mod helpers;
mod pushes;
mod report;
mod store_read_first;

use std::cell::RefCell;
use std::path::PathBuf;

use dam_adapters::LuaConfigFile;
use dam_application::testing::{ScriptedHelper, task_caps};
use dam_application::{
    HelperError, HelperLauncher, PullOutcome, RemoteCapabilities, RemoteConfig, RemoteHelper,
    Secret,
};
use tempfile::TempDir;

use super::{Machine, SETUP, checks};
use crate::args::CheckName;
use crate::commands::check_report::CheckReport;

const FAKE_AND_WORK: &str =
    "return { remote = { fake = { url = \"fake::\" }, work = { url = \"nosuch::\" } } }\n";
const FAKE_ONLY: &str = "return { remote = { fake = { url = \"fake::\" } } }\n";
const FAKE_SILENT: &str = "return { remote = { fake = { url = \"silent::\" } } }\n";
const FAKE_TOKEN: &str = "return { schema = 1, remote = { fake = { api_token = \"tok-123\" } } }\n";

#[derive(Default)]
struct Helpers {
    credentials_given: RefCell<Vec<usize>>,
}

impl HelperLauncher for Helpers {
    fn launch(
        &self,
        remote: &RemoteConfig,
        credentials: &[(String, Secret)],
    ) -> Result<Box<dyn RemoteHelper>, HelperError> {
        self.credentials_given.borrow_mut().push(credentials.len());
        match remote.helper.as_str() {
            "fake" => Ok(Box::new(ScriptedHelper {
                caps: RemoteCapabilities {
                    credentials: vec!["api_token".into()],
                    ..task_caps()
                },
                ..ScriptedHelper::answering(Ok(PullOutcome::default()))
            })),
            "silent" => Err(HelperError::Timeout {
                helper: "fake".into(),
                deadline: "1s".into(),
                said: None,
            }),
            other => Err(HelperError::NotFound {
                helper: format!("dam-remote-{other}"),
            }),
        }
    }
}

struct Files {
    dir: TempDir,
    config: LuaConfigFile,
}

impl Files {
    fn new(config_lua: &str, secrets_lua: &str) -> Files {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.lua"), config_lua).unwrap();
        std::fs::write(dir.path().join("secrets.lua"), secrets_lua).unwrap();
        let config = LuaConfigFile::new(dir.path().join("config.lua"));
        Files { dir, config }
    }

    fn store_path(&self) -> PathBuf {
        self.dir.path().join("dam.db")
    }

    fn report(&self, launcher: &Helpers, names: &[CheckName]) -> CheckReport {
        let store = self.store_path();
        let machine = Machine::new(&self.config, &store, launcher);
        CheckReport::new("doctor", SETUP, checks(names, &machine))
    }
}

fn rows_of(report: &CheckReport) -> Vec<String> {
    let human = crate::view::report(report).unwrap().human;
    human
        .lines()
        .skip(3)
        .take_while(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn lines_from(report: &CheckReport, first: &str) -> Vec<String> {
    let human = crate::view::report(report).unwrap().human;
    human
        .lines()
        .skip_while(|line| *line != first)
        .map(str::to_string)
        .collect()
}

fn json(report: &CheckReport) -> serde_json::Value {
    serde_json::to_value(report).unwrap()
}
