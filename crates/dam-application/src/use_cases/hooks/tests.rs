mod accepting;
mod edits;
mod events;
mod failures;
mod progress;
mod quiet;
mod resolving;

use std::path::PathBuf;

use dam_domain::{Object, Path, Priority};

use crate::config::{Config, HookConfig, HookEvent, OnFailure};
use crate::ports::{FunctionFailure, FunctionFault};
use crate::testing::{Scenario, config, task_in};
use crate::use_cases::edit::EditFields;
use crate::use_cases::new_object::NewTask;

fn hooked(hooks: &[(HookEvent, Option<OnFailure>)]) -> Scenario {
    let s = Scenario::default().drawing_from(9).configured(Config {
        hooks: hooks
            .iter()
            .map(|&(event, on_failure)| HookConfig { event, on_failure })
            .collect(),
        ..config()
    });
    s.committed(Object::Task(task_in(1, "pay rent", "home")));
    s
}

fn every_hook() -> Scenario {
    hooked(&HookEvent::LOCAL.map(|event| (event, None)))
}

fn new_task(subject: &str) -> NewTask {
    NewTask {
        parent: None,
        subject: subject.into(),
        path: Path::root(),
        priority: Priority::default(),
        due: None,
        deadline: None,
        labels: Default::default(),
        body: String::new(),
        id: None,
    }
}

fn retitled(subject: &str) -> EditFields {
    EditFields {
        subject: Some(subject.into()),
        ..EditFields::default()
    }
}

fn raised() -> FunctionFailure {
    FunctionFailure {
        file: PathBuf::from("/home/ada/.config/dam/config.lua"),
        line: Some(4),
        fault: FunctionFault::Raised {
            message: "attempt to call a nil value".into(),
        },
    }
}

fn ran(s: &Scenario) -> Vec<(HookEvent, &'static str)> {
    s.fixture
        .hooks
        .ran()
        .into_iter()
        .map(|(event, subject)| (event, &*subject.leak()))
        .collect()
}
