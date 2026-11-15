mod plist;

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use dam_application::{Installed, Removed, Schedule, ScheduleError, ScheduledRun};

use super::ProgramRunner;
use super::files::{ran, remove_each_present, write_when_changed};

const LABEL: &str = "dam-sync";

pub struct Launchd<R: ProgramRunner> {
    pub runner: R,
    pub home: PathBuf,
}

impl<R: ProgramRunner> Launchd<R> {
    fn plist_path(&self) -> PathBuf {
        self.home
            .join("Library/LaunchAgents")
            .join(format!("{LABEL}.plist"))
    }

    fn launchctl(&self, verb: &str, target: &str) -> Result<(), ScheduleError> {
        ran(
            "launchctl",
            verb,
            self.runner.run("launchctl", &[verb, target]),
        )
    }

    fn loaded(&self, service: &str) -> bool {
        self.runner.run("launchctl", &["print", service]).is_ok()
    }
}

fn domain_of_the_owner_of(plist: &Path) -> Result<String, ScheduleError> {
    std::fs::metadata(plist)
        .map(|meta| format!("gui/{}", meta.uid()))
        .map_err(|e| ScheduleError::Failed(format!("could not read {}: {e}", plist.display())))
}

impl<R: ProgramRunner> Schedule for Launchd<R> {
    fn installed(&self) -> bool {
        self.plist_path().exists()
    }

    fn install(&self, run: &ScheduledRun) -> Result<Installed, ScheduleError> {
        let path = self.plist_path();
        let changed = write_when_changed(&path, &plist::plist(LABEL, run))?;
        let domain = domain_of_the_owner_of(&path)?;
        let service = format!("{domain}/{LABEL}");
        let loaded = self.loaded(&service);
        if loaded && !changed {
            return Ok(Installed::Already(vec![path]));
        }
        if loaded {
            self.launchctl("bootout", &service)?;
        }
        let bootstrap = ["bootstrap", domain.as_str(), &path.display().to_string()];
        ran(
            "launchctl",
            "bootstrap",
            self.runner.run("launchctl", &bootstrap),
        )?;
        Ok(Installed::Now(vec![path]))
    }

    fn remove(&self) -> Result<Removed, ScheduleError> {
        let path = self.plist_path();
        if !path.exists() {
            return Ok(Removed::NotInstalled);
        }
        let service = format!("{}/{LABEL}", domain_of_the_owner_of(&path)?);
        if self.loaded(&service) {
            self.launchctl("bootout", &service)?;
        }
        Ok(Removed::Now(remove_each_present(&[path])?))
    }
}

#[cfg(test)]
mod tests;
