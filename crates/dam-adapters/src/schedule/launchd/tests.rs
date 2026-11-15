use std::os::unix::fs::MetadataExt;

use dam_application::testing::contract::schedule_contract;
use dam_application::{Installed, Removed, Schedule, ScheduleError};

use super::Launchd;
use dam_application::testing::contract::scheduled_run;

use crate::schedule::testing::RecordingRunner;

fn launchd(home: &tempfile::TempDir, runner: RecordingRunner) -> Launchd<RecordingRunner> {
    Launchd {
        runner,
        home: home.path().to_path_buf(),
    }
}

fn domain(home: &tempfile::TempDir) -> String {
    format!("gui/{}", std::fs::metadata(home.path()).unwrap().uid())
}

#[test]
fn the_launchd_schedule_keeps_the_schedule_contract() {
    let home = tempfile::tempdir().unwrap();
    let schedule = launchd(&home, RecordingRunner::default());
    schedule_contract(&schedule, &scheduled_run());
}

#[test]
fn the_agent_runs_dam_sync_run_every_minute_with_the_paths_it_was_enabled_with() {
    let home = tempfile::tempdir().unwrap();
    let plist = home.path().join("Library/LaunchAgents/dam-sync.plist");
    let mut run = scheduled_run();
    run.store = "/home/ada/a&b<c>.db".into();
    run.state_home = "/home/ada/state & more".into();

    launchd(&home, RecordingRunner::default())
        .install(&run)
        .unwrap();

    let expected = include_str!("expected.plist");
    assert_eq!(std::fs::read_to_string(plist).unwrap(), expected);
}

#[test]
fn install_bootstraps_the_agent_and_a_second_install_leaves_it_running() {
    let home = tempfile::tempdir().unwrap();
    let schedule = launchd(&home, RecordingRunner::default());
    let plist = home.path().join("Library/LaunchAgents/dam-sync.plist");
    let mut run = scheduled_run();

    assert_eq!(
        schedule.install(&run),
        Ok(Installed::Now(vec![plist.clone()]))
    );
    let domain = domain(&home);
    let service = format!("{domain}/dam-sync");
    assert_eq!(
        schedule.runner.calls(),
        [
            format!("launchctl print {service}"),
            format!("launchctl bootstrap {domain} {}", plist.display()),
        ]
    );

    assert_eq!(
        schedule.install(&run),
        Ok(Installed::Already(vec![plist.clone()]))
    );
    assert_eq!(
        schedule.runner.calls(),
        [format!("launchctl print {service}")]
    );

    run.store = "/home/ada/elsewhere.db".into();
    assert_eq!(
        schedule.install(&run),
        Ok(Installed::Now(vec![plist.clone()]))
    );
    assert_eq!(
        schedule.runner.calls(),
        [
            format!("launchctl print {service}"),
            format!("launchctl bootout {service}"),
            format!("launchctl bootstrap {domain} {}", plist.display()),
        ]
    );
}

#[test]
fn remove_boots_the_agent_out_and_deletes_its_file() {
    let home = tempfile::tempdir().unwrap();
    let schedule = launchd(&home, RecordingRunner::default());
    let plist = home.path().join("Library/LaunchAgents/dam-sync.plist");
    schedule.install(&scheduled_run()).unwrap();
    schedule.runner.calls();

    assert_eq!(schedule.remove(), Ok(Removed::Now(vec![plist.clone()])));
    let service = format!("{}/dam-sync", domain(&home));
    assert_eq!(
        schedule.runner.calls(),
        [
            format!("launchctl print {service}"),
            format!("launchctl bootout {service}"),
        ]
    );
    assert!(!plist.exists());
}

#[test]
fn a_refused_bootstrap_is_a_failure_that_says_why() {
    let home = tempfile::tempdir().unwrap();
    let schedule = launchd(
        &home,
        RecordingRunner::refusing_to_start("Bootstrap failed: 5"),
    );
    assert_eq!(
        schedule.install(&scheduled_run()),
        Err(ScheduleError::Failed(
            "launchctl bootstrap: Bootstrap failed: 5".into()
        ))
    );
}
