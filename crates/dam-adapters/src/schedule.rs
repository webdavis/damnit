mod crontab;
mod files;
mod launchd;
mod runner;
mod systemd;

use dam_application::Schedule;

use crontab::NoScheduler;
use launchd::Launchd;
use runner::{ProgramRunner, SystemRunner};
use systemd::Systemd;

pub fn this_systems_schedule() -> Box<dyn Schedule> {
    if cfg!(target_os = "macos") {
        let home = crate::paths::home_dir();
        return Box::new(Launchd {
            runner: SystemRunner,
            home,
        });
    }
    if std::path::Path::new("/run/systemd/system").is_dir() {
        let config_home = crate::paths::config_home();
        return Box::new(Systemd {
            runner: SystemRunner,
            config_home,
        });
    }
    Box::new(NoScheduler)
}

#[cfg(test)]
mod testing;
