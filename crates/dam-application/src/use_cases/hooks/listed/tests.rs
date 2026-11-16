mod calls;
mod changes;
mod failures;
mod limits;
mod progress;

use std::path::PathBuf;

use dam_domain::{Object, Oid};

use crate::config::{Config, HookConfig, HookEvent, OnFailure, PulledChanges, PulledHook};
use crate::ports::{EditRequest, FunctionFailure, FunctionFault, ObjectRepository};
use crate::testing::{Scenario, config, task_in};
use crate::use_cases::edit::EditFields;

fn hooked(local: &[(HookEvent, Option<OnFailure>)], pulled: Option<PulledChanges>) -> Scenario {
    let s = Scenario::default().drawing_from(9).configured(Config {
        hooks: local
            .iter()
            .map(|&(event, on_failure)| HookConfig {
                event,
                on_failure,
                file: format!("hooks/{}.lua", event.as_str()),
            })
            .collect(),
        task_pulled: pulled.map(|changes| PulledHook {
            changes: Some(changes),
            file: "hooks/task_pulled.lua".into(),
        }),
        ..config()
    });
    s.committed(Object::Task(task_in(1, "pay rent", "home")));
    s.committed(Object::Task(task_in(2, "call Ann", "home")));
    s.committed(Object::Task(task_in(3, "buy milk", "errands")));
    s
}

fn every_hook() -> Scenario {
    let local = HookEvent::LOCAL.map(|event| (event, None));
    hooked(&local, Some(PulledChanges::Committed))
}

fn requesting(s: &Scenario, subject: &str) {
    *s.fixture.hooks.requesting.borrow_mut() = Some(EditRequest {
        json: String::new(),
        file: PathBuf::from("hooks/task_new.lua"),
        line: Some(3),
    });
    *s.fixture.edit_requests.reading.borrow_mut() = Some(Ok(EditFields {
        subject: Some(subject.into()),
        ..EditFields::default()
    }));
}

fn raised() -> FunctionFailure {
    FunctionFailure {
        file: PathBuf::from("hooks/task_changed.lua"),
        line: Some(2),
        fault: FunctionFault::Raised {
            message: "no bell".into(),
        },
    }
}

fn ran(s: &Scenario) -> Vec<(HookEvent, String)> {
    s.fixture.hooks.ran()
}

fn subject(s: &Scenario, oid: &Oid) -> String {
    s.store.get(oid).unwrap().unwrap().base().subject.clone()
}
