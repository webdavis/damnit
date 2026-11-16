mod unit;

pub(super) use unit::unit_name;

use std::path::{Path, PathBuf};

use dam_application::{BackgroundJob, BackgroundState, BackgroundSync, Change, ScheduleError};

use super::ProgramRunner;
use super::files::{ran, remove_each_present, write_when_changed};

pub(super) struct Systemd<R: ProgramRunner> {
    pub runner: R,
    pub config_home: PathBuf,
    pub runtime: PathBuf,
}

const NOT_ACTIVE: i32 = 3;
const NO_SUCH_UNIT: i32 = 4;

impl<R: ProgramRunner> Systemd<R> {
    fn enabled_folder(&self) -> PathBuf {
        self.config_home.join("systemd/user")
    }

    fn started_folder(&self) -> PathBuf {
        self.runtime.join("systemd/user")
    }

    fn active(&self, timer: &str) -> Result<bool, ScheduleError> {
        let args = ["--user", "is-active", "--quiet", timer];
        match self.runner.run("systemctl", &args) {
            Ok(()) => Ok(true),
            Err(failure) if matches!(failure.code, Some(NOT_ACTIVE | NO_SUCH_UNIT)) => Ok(false),
            Err(failure) => ran("systemctl", "is-active", Err(failure)).map(|()| false),
        }
    }

    fn refuse_what_the_enabled_units_overrule(
        &self,
        job: &BackgroundJob,
    ) -> Result<(), ScheduleError> {
        let enabled = self
            .enabled_folder()
            .join(unit_name(&job.remote, "service"));
        match std::fs::read_to_string(enabled) {
            Ok(held) if held != unit::service(job) => Err(ScheduleError::Failed(format!(
                "{0} starts at login with other settings, which systemd keeps in force over these; run dam sync disable {0} first",
                job.remote
            ))),
            _ => Ok(()),
        }
    }

    fn systemctl(&self, verb: &str, unit: Option<&str>) -> Result<(), ScheduleError> {
        let args: Vec<&str> = ["--user", verb].into_iter().chain(unit).collect();
        ran("systemctl", verb, self.runner.run("systemctl", &args))
    }
}

fn write_units(folder: &Path, job: &BackgroundJob) -> Result<bool, ScheduleError> {
    let service = write_when_changed(
        &folder.join(unit_name(&job.remote, "service")),
        &unit::service(job),
    )?;
    let timer = write_when_changed(
        &folder.join(unit_name(&job.remote, "timer")),
        &unit::timer(&job.remote),
    )?;
    Ok(service || timer)
}

fn units_in(folder: &Path, remote: &str) -> [PathBuf; 2] {
    ["timer", "service"].map(|kind| folder.join(unit_name(remote, kind)))
}

fn changed(now: bool) -> Change {
    match now {
        true => Change::Now,
        false => Change::Already,
    }
}

impl<R: ProgramRunner> BackgroundSync for Systemd<R> {
    fn start(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        self.refuse_what_the_enabled_units_overrule(job)?;
        let rewritten = write_units(&self.started_folder(), job)?;
        if rewritten {
            self.systemctl("daemon-reload", None)?;
        }
        let timer = unit_name(&job.remote, "timer");
        if self.active(&timer)? {
            return Ok(changed(rewritten));
        }
        self.systemctl("start", Some(&timer))?;
        Ok(Change::Now)
    }

    fn stop(&self, remote: &str) -> Result<Change, ScheduleError> {
        let timer = unit_name(remote, "timer");
        if !self.active(&timer)? {
            return Ok(Change::Already);
        }
        self.systemctl("stop", Some(&timer))?;
        Ok(Change::Now)
    }

    fn enable(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        if !write_units(&self.enabled_folder(), job)? {
            return Ok(Change::Already);
        }
        self.systemctl("enable", Some(&unit_name(&job.remote, "timer")))?;
        Ok(Change::Now)
    }

    fn disable(&self, remote: &str) -> Result<Change, ScheduleError> {
        let units = units_in(&self.enabled_folder(), remote);
        if !units.iter().any(|unit| unit.exists()) {
            return Ok(Change::Already);
        }
        self.systemctl("disable", Some(&unit_name(remote, "timer")))?;
        remove_each_present(&units)?;
        Ok(Change::Now)
    }

    fn state(&self, remote: &str) -> Result<BackgroundState, ScheduleError> {
        Ok(BackgroundState {
            running: self.active(&unit_name(remote, "timer"))?,
            at_login: units_in(&self.enabled_folder(), remote)[0].exists(),
        })
    }
}

#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod tests;
