use std::cell::Cell;
use std::path::PathBuf;

use crate::ports::{Installed, Removed, Schedule, ScheduleError, ScheduledRun};

#[derive(Default)]
pub struct FakeSchedule {
    installed: Cell<bool>,
}

impl FakeSchedule {
    fn files() -> Vec<PathBuf> {
        vec![PathBuf::from("/fake/schedule")]
    }
}

impl Schedule for FakeSchedule {
    fn installed(&self) -> bool {
        self.installed.get()
    }

    fn install(&self, _: &ScheduledRun) -> Result<Installed, ScheduleError> {
        match self.installed.replace(true) {
            true => Ok(Installed::Already(Self::files())),
            false => Ok(Installed::Now(Self::files())),
        }
    }

    fn remove(&self) -> Result<Removed, ScheduleError> {
        match self.installed.replace(false) {
            true => Ok(Removed::Now(Self::files())),
            false => Ok(Removed::NotInstalled),
        }
    }
}

#[cfg(test)]
mod tests;
