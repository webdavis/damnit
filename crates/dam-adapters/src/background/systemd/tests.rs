use std::path::PathBuf;
use std::time::Duration;

use dam_application::{BackgroundSync, Change, ScheduleError};

use super::Systemd;
use crate::background::testing::{RecordingRunner, job};

struct Machine {
    config: tempfile::TempDir,
    runtime: tempfile::TempDir,
}

impl Machine {
    fn new() -> Machine {
        Machine {
            config: tempfile::tempdir().unwrap(),
            runtime: tempfile::tempdir().unwrap(),
        }
    }

    fn systemd(&self) -> Systemd<RecordingRunner> {
        Systemd {
            runner: RecordingRunner::default(),
            config_home: self.config.path().to_path_buf(),
            runtime: self.runtime.path().to_path_buf(),
        }
    }

    fn enabled(&self, kind: &str) -> PathBuf {
        let name = format!("systemd/user/dam-sync-todoist.{kind}");
        self.config.path().join(name)
    }

    fn started(&self, kind: &str) -> PathBuf {
        let name = format!("systemd/user/dam-sync-todoist.{kind}");
        self.runtime.path().join(name)
    }
}

const RELOAD: &str = "systemctl --user daemon-reload";
const IS_ACTIVE: &str = "systemctl --user is-active --quiet dam-sync-todoist.timer";

#[test]
fn start_writes_runtime_units_and_starts_the_timer_and_a_second_start_leaves_it() {
    let m = Machine::new();
    let systemd = m.systemd();
    assert_eq!(systemd.start(&job()), Ok(Change::Now));
    assert_eq!(
        systemd.runner.calls(),
        [
            RELOAD,
            IS_ACTIVE,
            "systemctl --user start dam-sync-todoist.timer"
        ]
    );
    let service = std::fs::read_to_string(m.started("service")).unwrap();
    assert_eq!(service, include_str!("expected.service"));
    let timer = std::fs::read_to_string(m.started("timer")).unwrap();
    assert_eq!(timer, include_str!("expected.timer"));
    assert!(!m.enabled("timer").exists());
    assert_eq!(systemd.start(&job()), Ok(Change::Already));
    assert_eq!(systemd.runner.calls(), [IS_ACTIVE]);
}

#[test]
fn start_with_a_new_interval_reloads_the_running_timer() {
    let m = Machine::new();
    let systemd = m.systemd();
    systemd.start(&job()).unwrap();
    systemd.runner.calls();
    let mut faster = job();
    faster.interval = Some(Duration::from_secs(300));
    assert_eq!(systemd.start(&faster), Ok(Change::Now));
    assert_eq!(systemd.runner.calls(), [RELOAD, IS_ACTIVE]);
    let service = std::fs::read_to_string(m.started("service")).unwrap();
    assert!(service.contains(r#""--when-due" "--interval" "300s""#));
}

#[test]
fn stop_stops_the_timer_and_leaves_the_enabled_units() {
    let m = Machine::new();
    let systemd = m.systemd();
    systemd.enable(&job()).unwrap();
    systemd.start(&job()).unwrap();
    systemd.runner.calls();
    assert_eq!(systemd.stop("todoist"), Ok(Change::Now));
    assert_eq!(
        systemd.runner.calls(),
        [IS_ACTIVE, "systemctl --user stop dam-sync-todoist.timer"]
    );
    assert!(m.enabled("timer").exists());
    assert_eq!(systemd.stop("todoist"), Ok(Change::Already));
}

#[test]
fn enable_writes_units_for_the_next_login_and_starts_nothing() {
    let m = Machine::new();
    let systemd = m.systemd();
    assert_eq!(systemd.enable(&job()), Ok(Change::Now));
    assert_eq!(
        systemd.runner.calls(),
        ["systemctl --user enable dam-sync-todoist.timer"]
    );
    let timer = std::fs::read_to_string(m.enabled("timer")).unwrap();
    assert_eq!(timer, include_str!("expected.timer"));
    assert!(m.enabled("service").exists());
    assert!(!systemd.runner.loaded.get());
    assert_eq!(systemd.enable(&job()), Ok(Change::Already));
    assert_eq!(systemd.runner.calls(), Vec::<String>::new());
}

#[test]
fn disable_removes_the_enabled_units_and_stops_nothing() {
    let m = Machine::new();
    let systemd = m.systemd();
    systemd.enable(&job()).unwrap();
    systemd.start(&job()).unwrap();
    systemd.runner.calls();
    assert_eq!(systemd.disable("todoist"), Ok(Change::Now));
    assert_eq!(
        systemd.runner.calls(),
        ["systemctl --user disable dam-sync-todoist.timer"]
    );
    assert!(!m.enabled("timer").exists());
    assert!(!m.enabled("service").exists());
    assert!(systemd.runner.loaded.get());
    assert_eq!(systemd.disable("todoist"), Ok(Change::Already));
    assert_eq!(systemd.runner.calls(), Vec::<String>::new());
}

#[test]
fn a_failing_systemctl_is_reported_and_starts_nothing() {
    let m = Machine::new();
    let systemd = m.systemd();
    systemd.runner.broken.replace(Some("--user"));
    assert_eq!(
        systemd.start(&job()),
        Err(ScheduleError::Failed(
            "systemctl daemon-reload: refused".into()
        ))
    );
    assert!(matches!(
        systemd.enable(&job()),
        Err(ScheduleError::Failed(_))
    ));
    assert!(!systemd.runner.loaded.get());
}

#[test]
fn a_remote_name_outside_systemds_unit_name_rules_is_escaped() {
    assert_eq!(
        super::unit_name("my list", "timer"),
        r"dam-sync-my\x20list.timer"
    );
}
