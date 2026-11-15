use dam_application::testing::contract::schedule_contract;
use dam_application::{Installed, Removed, Schedule, ScheduleError};

use super::Systemd;
use dam_application::testing::contract::scheduled_run;

use crate::schedule::testing::RecordingRunner;

fn systemd(config_home: &tempfile::TempDir, runner: RecordingRunner) -> Systemd<RecordingRunner> {
    Systemd {
        runner,
        config_home: config_home.path().to_path_buf(),
    }
}

fn units(config_home: &tempfile::TempDir) -> Vec<std::path::PathBuf> {
    let folder = config_home.path().join("systemd/user");
    vec![
        folder.join("dam-sync.service"),
        folder.join("dam-sync.timer"),
    ]
}

#[test]
fn the_systemd_schedule_keeps_the_schedule_contract() {
    let config_home = tempfile::tempdir().unwrap();
    let schedule = systemd(&config_home, RecordingRunner::default());
    schedule_contract(&schedule, &scheduled_run());
}

#[test]
fn the_timer_starts_the_service_every_minute_with_the_paths_it_was_enabled_with() {
    let config_home = tempfile::tempdir().unwrap();
    let mut run = scheduled_run();
    run.store = "/home/ada/100% \"$HOME\".db".into();
    run.state_home = "/home/ada/100% state".into();

    systemd(&config_home, RecordingRunner::default())
        .install(&run)
        .unwrap();

    let [service, timer] = [0, 1].map(|i| std::fs::read_to_string(&units(&config_home)[i]));
    let expected = include_str!("expected.service");
    assert_eq!(service.unwrap(), expected);
    assert_eq!(timer.unwrap(), include_str!("expected.timer"));
}

#[test]
fn install_enables_the_timer_and_a_second_install_leaves_it_running() {
    let config_home = tempfile::tempdir().unwrap();
    let schedule = systemd(&config_home, RecordingRunner::default());
    let run = scheduled_run();

    assert_eq!(
        schedule.install(&run),
        Ok(Installed::Now(units(&config_home)))
    );
    assert_eq!(
        schedule.runner.calls(),
        [
            "systemctl --user is-active --quiet dam-sync.timer",
            "systemctl --user daemon-reload",
            "systemctl --user enable --now dam-sync.timer",
        ]
    );

    assert_eq!(
        schedule.install(&run),
        Ok(Installed::Already(units(&config_home)))
    );
    assert_eq!(
        schedule.runner.calls(),
        ["systemctl --user is-active --quiet dam-sync.timer"]
    );
}

#[test]
fn remove_disables_the_timer_and_deletes_both_units() {
    let config_home = tempfile::tempdir().unwrap();
    let schedule = systemd(&config_home, RecordingRunner::default());
    schedule.install(&scheduled_run()).unwrap();
    schedule.runner.calls();

    assert_eq!(schedule.remove(), Ok(Removed::Now(units(&config_home))));
    assert_eq!(
        schedule.runner.calls(),
        [
            "systemctl --user disable --now dam-sync.timer",
            "systemctl --user daemon-reload",
        ]
    );
    assert!(units(&config_home).iter().all(|unit| !unit.exists()));
}

#[test]
fn a_refused_enable_is_a_failure_that_says_why() {
    let config_home = tempfile::tempdir().unwrap();
    let schedule = systemd(
        &config_home,
        RecordingRunner::refusing_to_start("No such bus"),
    );
    assert_eq!(
        schedule.install(&scheduled_run()),
        Err(ScheduleError::Failed(
            "systemctl enable: No such bus".into()
        ))
    );
}
