mod files;
mod launchd;
mod no_scheduler;
mod runner;

use std::path::{Path, PathBuf};

use dam_application::BackgroundSync;

use launchd::Launchd;
use no_scheduler::NoScheduler;
use runner::{ProgramRunner, SystemRunner};

pub fn background_sync_for_store(store: &Path) -> Box<dyn BackgroundSync> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let state = store.parent().unwrap_or(Path::new(".")).join("background");
    match cfg!(target_os = "macos") {
        true => Box::new(Launchd {
            runner: SystemRunner,
            home,
            state,
        }),
        false => Box::new(NoScheduler),
    }
}

pub fn no_background_sync() -> Box<dyn BackgroundSync> {
    Box::new(NoScheduler)
}

#[cfg(test)]
mod testing;
