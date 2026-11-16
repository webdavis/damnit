mod files;
mod launchd;
mod runner;

use std::path::PathBuf;

use dam_application::BackgroundSync;

use launchd::Launchd;
use runner::{ProgramRunner, SystemRunner};

pub fn launchd_background_sync(home: PathBuf, state: PathBuf) -> Box<dyn BackgroundSync> {
    Box::new(Launchd {
        runner: SystemRunner,
        home,
        state,
    })
}

#[cfg(test)]
mod testing;
