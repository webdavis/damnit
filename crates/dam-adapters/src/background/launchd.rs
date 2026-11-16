mod plist;

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use dam_application::{BackgroundJob, BackgroundSync, Change, ScheduleError, label_of};

use super::ProgramRunner;
use super::files::{ran, remove_each_present, write_when_changed};
use super::runner::RunFailure;

pub struct Launchd<R: ProgramRunner> {
    pub runner: R,
    pub home: PathBuf,
    pub state: PathBuf,
}

impl<R: ProgramRunner> Launchd<R> {
    fn enabled_agent(&self, remote: &str) -> PathBuf {
        agent_in(&self.home.join("Library/LaunchAgents"), remote)
    }

    fn domain(&self) -> Result<String, ScheduleError> {
        std::fs::metadata(&self.home)
            .map(|meta| format!("gui/{}", meta.uid()))
            .map_err(|e| {
                ScheduleError::Failed(format!("could not read {}: {e}", self.home.display()))
            })
    }

    fn service(&self, remote: &str) -> Result<String, ScheduleError> {
        Ok(format!("{}/{}", self.domain()?, label_of(remote)))
    }

    fn loaded(&self, service: &str) -> Result<bool, ScheduleError> {
        match self.runner.run("launchctl", &["print", service]) {
            Ok(()) => Ok(true),
            Err(failure) if says_not_loaded(&failure) => Ok(false),
            Err(failure) => ran("launchctl", "print", Err(failure)).map(|()| false),
        }
    }

    fn launchctl(&self, args: &[&str]) -> Result<(), ScheduleError> {
        ran("launchctl", args[0], self.runner.run("launchctl", args))
    }
}

const SERVICE_NOT_FOUND: i32 = 113;
const COULD_NOT_FIND: &str = "Could not find service";

fn says_not_loaded(failure: &RunFailure) -> bool {
    failure.code == Some(SERVICE_NOT_FOUND) || failure.message.contains(COULD_NOT_FIND)
}

fn agent_in(folder: &Path, remote: &str) -> PathBuf {
    folder.join(format!("{}.plist", label_of(remote)))
}

fn changed(now: bool) -> Change {
    match now {
        true => Change::Now,
        false => Change::Already,
    }
}

impl<R: ProgramRunner> BackgroundSync for Launchd<R> {
    fn start(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        let path = agent_in(&self.state, &job.remote);
        let rewritten = write_when_changed(&path, &plist::plist(job))?;
        let service = self.service(&job.remote)?;
        let loaded = self.loaded(&service)?;
        if loaded && !rewritten {
            return Ok(Change::Already);
        }
        if loaded {
            self.launchctl(&["bootout", &service])?;
        }
        let path = path.display().to_string();
        self.launchctl(&["bootstrap", &self.domain()?, &path])?;
        Ok(Change::Now)
    }

    fn stop(&self, remote: &str) -> Result<Change, ScheduleError> {
        let service = self.service(remote)?;
        if !self.loaded(&service)? {
            return Ok(Change::Already);
        }
        self.launchctl(&["bootout", &service])?;
        Ok(Change::Now)
    }

    fn enable(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        let path = self.enabled_agent(&job.remote);
        Ok(changed(write_when_changed(&path, &plist::plist(job))?))
    }

    fn disable(&self, remote: &str) -> Result<Change, ScheduleError> {
        let removed = remove_each_present(&[self.enabled_agent(remote)])?;
        Ok(changed(!removed.is_empty()))
    }
}

#[cfg(test)]
mod tests;
