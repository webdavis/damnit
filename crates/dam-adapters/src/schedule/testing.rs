use std::cell::{Cell, RefCell};

use super::ProgramRunner;

#[derive(Default)]
pub(super) struct RecordingRunner {
    calls: RefCell<Vec<String>>,
    loaded: Cell<bool>,
    refusal: Option<&'static str>,
}

impl RecordingRunner {
    pub(super) fn refusing_to_start(why: &'static str) -> RecordingRunner {
        RecordingRunner {
            refusal: Some(why),
            ..RecordingRunner::default()
        }
    }

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
            ["bootstrap", ..] | ["--user", "enable", ..] => match self.refusal {
                Some(why) => Err(why.into()),
                None => {
                    self.loaded.set(true);
                    Ok(())
                }
            },
            ["bootout", ..] | ["--user", "disable", ..] => {
                self.loaded.set(false);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
