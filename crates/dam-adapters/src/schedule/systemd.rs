mod units;

use std::path::PathBuf;

use dam_application::{Installed, Removed, Schedule, ScheduleError, ScheduledRun};

use super::ProgramRunner;
use super::files::{ran, remove_each_present, write_when_changed};

const TIMER: &str = "dam-sync.timer";

pub struct Systemd<R: ProgramRunner> {
    pub runner: R,
    pub config_home: PathBuf,
}

impl<R: ProgramRunner> Systemd<R> {
    fn units(&self) -> [PathBuf; 2] {
        let folder = self.config_home.join("systemd/user");
        [folder.join("dam-sync.service"), folder.join(TIMER)]
    }

    fn systemctl(&self, args: &[&str]) -> Result<(), String> {
        self.runner.run("systemctl", &[&["--user"], args].concat())
    }
}

impl<R: ProgramRunner> Schedule for Systemd<R> {
    fn installed(&self) -> bool {
        self.units()[1].exists()
    }

    fn install(&self, run: &ScheduledRun) -> Result<Installed, ScheduleError> {
        let [service, timer] = self.units();
        let service_changed = write_when_changed(&service, &units::service(run))?;
        let timer_changed = write_when_changed(&timer, units::TIMER)?;
        let active = self.systemctl(&["is-active", "--quiet", TIMER]).is_ok();
        let files = vec![service, timer];
        if active && !service_changed && !timer_changed {
            return Ok(Installed::Already(files));
        }
        ran(
            "systemctl",
            "daemon-reload",
            self.systemctl(&["daemon-reload"]),
        )?;
        ran(
            "systemctl",
            "enable",
            self.systemctl(&["enable", "--now", TIMER]),
        )?;
        Ok(Installed::Now(files))
    }

    fn remove(&self) -> Result<Removed, ScheduleError> {
        if !self.units().iter().any(|unit| unit.exists()) {
            return Ok(Removed::NotInstalled);
        }
        ran(
            "systemctl",
            "disable",
            self.systemctl(&["disable", "--now", TIMER]),
        )?;
        let removed = remove_each_present(&self.units())?;
        ran(
            "systemctl",
            "daemon-reload",
            self.systemctl(&["daemon-reload"]),
        )?;
        Ok(Removed::Now(removed))
    }
}

#[cfg(test)]
mod tests;
