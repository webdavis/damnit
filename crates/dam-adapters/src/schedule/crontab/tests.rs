use dam_application::testing::contract::scheduled_run;
use dam_application::{Removed, Schedule, ScheduleError};

use super::NoScheduler;

#[test]
fn install_refuses_and_gives_the_crontab_line_that_runs_dam_sync_run_every_minute() {
    let mut run = scheduled_run();
    run.store = "/home/ada/100% Ada's.db".into();
    let crontab_line = "* * * * * PATH='/opt/dam bin:/usr/bin' \
        XDG_STATE_HOME='/home/ada/state' '/opt/dam bin/dam' '--config' \
        '/home/ada/dam/config.lua' '--store' '/home/ada/100\\% Ada'\\''s.db' 'sync' 'run' \
        '--record-last-run' >/dev/null 2>&1";
    assert_eq!(
        NoScheduler.install(&run),
        Err(ScheduleError::NoScheduler {
            crontab_line: crontab_line.into()
        })
    );
}

#[test]
fn nothing_is_ever_installed_so_nothing_is_removed() {
    assert!(!NoScheduler.installed());
    assert_eq!(NoScheduler.remove(), Ok(Removed::NotInstalled));
}
