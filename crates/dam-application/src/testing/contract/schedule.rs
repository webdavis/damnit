use crate::ports::{Installed, Removed, Schedule, ScheduledRun};

pub fn scheduled_run() -> ScheduledRun {
    ScheduledRun {
        dam: "/opt/dam bin/dam".into(),
        config: "/home/ada/dam/config.lua".into(),
        store: "/home/ada/dam/dam.db".into(),
        state_home: "/home/ada/state".into(),
        search_path: "/opt/dam bin:/usr/bin".into(),
    }
}

pub fn schedule_contract(schedule: &dyn Schedule, run: &ScheduledRun) {
    assert!(!schedule.installed(), "a fresh schedule is not installed");
    assert_eq!(schedule.remove(), Ok(Removed::NotInstalled));

    let Ok(Installed::Now(files)) = schedule.install(run) else {
        panic!("the first install installs");
    };
    assert!(!files.is_empty(), "an install names what it wrote");
    assert!(schedule.installed());
    assert_eq!(schedule.install(run), Ok(Installed::Already(files.clone())));

    assert_eq!(schedule.remove(), Ok(Removed::Now(files)));
    assert!(!schedule.installed());
    assert_eq!(schedule.remove(), Ok(Removed::NotInstalled));
}
