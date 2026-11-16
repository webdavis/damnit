use std::cell::{Cell, RefCell};

use dam_application::BackgroundJob;

use super::ProgramRunner;

#[derive(Default)]
pub(super) struct RecordingRunner {
    calls: RefCell<Vec<String>>,
    pub(super) loaded: Cell<bool>,
}

impl RecordingRunner {
    pub(super) fn calls(&self) -> Vec<String> {
        self.calls.take()
    }
}

impl ProgramRunner for RecordingRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), String> {
        let line = std::iter::once(program).chain(args.iter().copied());
        self.calls
            .borrow_mut()
            .push(line.collect::<Vec<_>>().join(" "));
        match args {
            ["print", ..] | ["--user", "is-active", ..] if !self.loaded.get() => {
                Err("not loaded".into())
            }
            ["bootstrap", ..] | ["--user", "start", ..] => {
                self.loaded.set(true);
                Ok(())
            }
            ["bootout", ..] | ["--user", "stop", ..] => {
                self.loaded.set(false);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub(super) fn job() -> BackgroundJob {
    BackgroundJob {
        remote: "todoist".into(),
        dam: "/opt/dam/bin/dam".into(),
        config: "/home/someone/config.lua".into(),
        store: "/home/someone/dam.db".into(),
        search_path: "/usr/bin:/bin".into(),
        interval: None,
    }
}
