mod files;
mod launchd;
mod no_scheduler;
mod run_record_files;
mod runner;
mod systemd;

use std::path::{Path, PathBuf};

use dam_application::{BackgroundSync, LastRun, RunRecords, ScheduleError};

use launchd::Launchd;
use no_scheduler::NoScheduler;
use run_record_files::RunRecordFiles;
use runner::{ProgramRunner, SystemRunner};
use systemd::Systemd;

pub fn background_sync_for_store(store: &Path) -> Box<dyn BackgroundSync> {
    let var = |name| std::env::var_os(name).map(PathBuf::from);
    let home = var("HOME").unwrap_or_default();
    match scheduler_on(cfg!(target_os = "macos"), var("XDG_RUNTIME_DIR")) {
        Scheduler::Launchd => Box::new(Launchd {
            runner: SystemRunner,
            state: state_folder(store),
            home,
        }),
        Scheduler::Systemd(runtime) => Box::new(Systemd {
            runner: SystemRunner,
            config_home: var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config")),
            runtime,
        }),
        Scheduler::None => Box::new(NoScheduler),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Scheduler {
    Launchd,
    Systemd(PathBuf),
    None,
}

fn scheduler_on(macos: bool, runtime: Option<PathBuf>) -> Scheduler {
    match runtime {
        _ if macos => Scheduler::Launchd,
        Some(runtime) if runtime.join("systemd").is_dir() => Scheduler::Systemd(runtime),
        _ => Scheduler::None,
    }
}

pub fn run_records_for_store(store: &Path) -> Box<dyn RunRecords> {
    Box::new(RunRecordFiles {
        folder: state_folder(store),
    })
}

pub fn no_run_records() -> Box<dyn RunRecords> {
    Box::new(NoRunRecords)
}

struct NoRunRecords;

impl RunRecords for NoRunRecords {
    fn record(&self, _: &str, _: &LastRun) -> Result<(), ScheduleError> {
        Ok(())
    }
    fn last(&self, _: &str) -> Result<Option<LastRun>, ScheduleError> {
        Ok(None)
    }
}

fn state_folder(store: &Path) -> PathBuf {
    store.parent().unwrap_or(Path::new(".")).join("background")
}

pub fn no_background_sync() -> Box<dyn BackgroundSync> {
    Box::new(NoScheduler)
}

#[cfg(test)]
mod scheduler_tests;
#[cfg(test)]
mod testing;
