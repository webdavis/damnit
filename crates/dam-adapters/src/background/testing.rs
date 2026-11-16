use std::cell::{Cell, RefCell};

use dam_application::BackgroundJob;

use super::runner::{ProgramRunner, RunFailure};

#[derive(Default)]
pub(super) struct RecordingRunner {
    calls: RefCell<Vec<String>>,
    pub(super) loaded: Cell<bool>,
    pub(super) broken: Cell<Option<&'static str>>,
    pub(super) says_not_loaded_as: Cell<Option<(i32, &'static str)>>,
}

impl RecordingRunner {
    pub(super) fn calls(&self) -> Vec<String> {
        self.calls.take()
    }
}

impl ProgramRunner for RecordingRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), RunFailure> {
        let line = std::iter::once(program).chain(args.iter().copied());
        self.calls
            .borrow_mut()
            .push(line.collect::<Vec<_>>().join(" "));
        if self
            .broken
            .get()
            .is_some_and(|verb| args.first() == Some(&verb))
        {
            return Err(exited(1, "refused"));
        }
        match args {
            ["print", ..] if !self.loaded.get() => {
                let (code, message) = self.says_not_loaded_as.get().unwrap_or((113, "not loaded"));
                Err(exited(code, message))
            }
            ["--user", "is-active", ..] if !self.loaded.get() => Err(exited(3, "inactive")),
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

fn exited(code: i32, message: &str) -> RunFailure {
    RunFailure {
        code: Some(code),
        message: message.to_string(),
    }
}
