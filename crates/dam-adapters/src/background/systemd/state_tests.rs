use std::time::Duration;

use dam_application::{BackgroundState, BackgroundSync, ScheduleError};

use super::Systemd;
use crate::background::testing::{RecordingRunner, job};

fn systemd(config: &tempfile::TempDir, runtime: &tempfile::TempDir) -> Systemd<RecordingRunner> {
    Systemd {
        runner: RecordingRunner::default(),
        config_home: config.path().to_path_buf(),
        runtime: runtime.path().to_path_buf(),
    }
}

#[test]
fn state_says_whether_the_timer_runs_and_whether_it_starts_at_login() {
    let (config, runtime) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let systemd = systemd(&config, &runtime);
    let state = |running, at_login| Ok(BackgroundState { running, at_login });
    assert_eq!(systemd.state("todoist"), state(false, false));
    systemd.enable(&job()).unwrap();
    assert_eq!(systemd.state("todoist"), state(false, true));
    systemd.start(&job()).unwrap();
    assert_eq!(systemd.state("todoist"), state(true, true));
    systemd.runner.calls();
    systemd.state("todoist").unwrap();
    assert_eq!(
        systemd.runner.calls(),
        ["systemctl --user is-active --quiet dam-sync-todoist.timer"]
    );
}

#[test]
fn start_refuses_settings_that_the_enabled_units_would_overrule() {
    let (config, runtime) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let systemd = systemd(&config, &runtime);
    systemd.enable(&job()).unwrap();
    systemd.runner.calls();
    let mut faster = job();
    faster.interval = Some(Duration::from_secs(300));
    let Err(ScheduleError::Failed(why)) = systemd.start(&faster) else {
        panic!("started")
    };
    assert!(why.contains("run dam sync disable todoist"), "{why}");
    assert_eq!(systemd.runner.calls(), Vec::<String>::new());
    assert!(systemd.start(&job()).is_ok());
}
