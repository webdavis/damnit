use std::os::unix::fs::MetadataExt;
use std::time::Duration;

use dam_application::{BackgroundSync, Change, ScheduleError};

use super::Launchd;
use crate::background::testing::{RecordingRunner, job};

struct Machine {
    home: tempfile::TempDir,
    state: tempfile::TempDir,
}

impl Machine {
    fn new() -> Machine {
        Machine {
            home: tempfile::tempdir().unwrap(),
            state: tempfile::tempdir().unwrap(),
        }
    }

    fn launchd(&self) -> Launchd<RecordingRunner> {
        Launchd {
            runner: RecordingRunner::default(),
            home: self.home.path().to_path_buf(),
            state: self.state.path().to_path_buf(),
        }
    }

    fn service(&self) -> String {
        let uid = std::fs::metadata(self.home.path()).unwrap().uid();
        format!("gui/{uid}/dam-sync.todoist")
    }

    fn enabled_agent(&self) -> std::path::PathBuf {
        self.home
            .path()
            .join("Library/LaunchAgents/dam-sync.todoist.plist")
    }

    fn started_agent(&self) -> std::path::PathBuf {
        self.state.path().join("dam-sync.todoist.plist")
    }
}

#[test]
fn start_loads_an_agent_kept_in_the_state_folder_and_a_second_start_leaves_it() {
    let m = Machine::new();
    let launchd = m.launchd();
    assert_eq!(launchd.start(&job()), Ok(Change::Now));
    let domain = m.service().rsplit_once('/').unwrap().0.to_string();
    assert_eq!(
        launchd.runner.calls(),
        [
            format!("launchctl print {}", m.service()),
            format!(
                "launchctl bootstrap {domain} {}",
                m.started_agent().display()
            ),
        ]
    );
    assert!(!m.enabled_agent().exists());
    assert_eq!(launchd.start(&job()), Ok(Change::Already));
}

#[test]
fn start_with_a_new_interval_reloads_the_agent() {
    let m = Machine::new();
    let launchd = m.launchd();
    launchd.start(&job()).unwrap();
    launchd.runner.calls();
    let mut faster = job();
    faster.interval = Some(Duration::from_secs(300));
    assert_eq!(launchd.start(&faster), Ok(Change::Now));
    let calls = launchd.runner.calls();
    assert_eq!(calls[1], format!("launchctl bootout {}", m.service()));
    let plist = std::fs::read_to_string(m.started_agent()).unwrap();
    assert!(plist.contains("<string>--interval</string>\n    <string>300s</string>"));
}

#[test]
fn stop_unloads_the_running_agent_and_leaves_the_enabled_one() {
    let m = Machine::new();
    let launchd = m.launchd();
    launchd.enable(&job()).unwrap();
    launchd.start(&job()).unwrap();
    launchd.runner.calls();
    assert_eq!(launchd.stop("todoist"), Ok(Change::Now));
    assert_eq!(
        launchd.runner.calls(),
        [
            format!("launchctl print {}", m.service()),
            format!("launchctl bootout {}", m.service()),
        ]
    );
    assert!(m.enabled_agent().exists());
    assert_eq!(launchd.stop("todoist"), Ok(Change::Already));
}

#[test]
fn enable_writes_the_agent_for_the_next_login_and_loads_nothing() {
    let m = Machine::new();
    let launchd = m.launchd();
    assert_eq!(launchd.enable(&job()), Ok(Change::Now));
    assert_eq!(launchd.runner.calls(), Vec::<String>::new());
    let plist = std::fs::read_to_string(m.enabled_agent()).unwrap();
    assert_eq!(plist, include_str!("expected.plist"));
    assert_eq!(launchd.enable(&job()), Ok(Change::Already));
}

#[test]
fn disable_removes_the_agent_and_stops_nothing() {
    let m = Machine::new();
    let launchd = m.launchd();
    launchd.enable(&job()).unwrap();
    launchd.start(&job()).unwrap();
    launchd.runner.calls();
    assert_eq!(launchd.disable("todoist"), Ok(Change::Now));
    assert_eq!(launchd.runner.calls(), Vec::<String>::new());
    assert!(!m.enabled_agent().exists());
    assert!(launchd.runner.loaded.get());
    assert_eq!(launchd.disable("todoist"), Ok(Change::Already));
}

#[test]
fn a_print_that_says_it_could_not_find_the_service_means_not_loaded_whatever_its_exit_code() {
    let m = Machine::new();
    let launchd = m.launchd();
    launchd.runner.says_not_loaded_as.replace(Some((
        64,
        "Could not find service \"dam-sync.todoist\" in domain for user gui: 501",
    )));
    assert_eq!(launchd.start(&job()), Ok(Change::Now));
    assert_eq!(launchd.runner.calls().len(), 2);
}

#[test]
fn a_failing_print_is_reported_and_loads_nothing() {
    let m = Machine::new();
    let launchd = m.launchd();
    launchd.runner.broken.replace(Some("print"));
    assert!(matches!(
        launchd.start(&job()),
        Err(ScheduleError::Failed(_))
    ));
    assert!(matches!(
        launchd.stop("todoist"),
        Err(ScheduleError::Failed(_))
    ));
    let calls = launchd.runner.calls();
    assert!(
        calls.iter().all(|c| c.starts_with("launchctl print")),
        "{calls:?}"
    );
}

#[test]
fn a_failing_bootout_is_reported() {
    let m = Machine::new();
    let launchd = m.launchd();
    launchd.start(&job()).unwrap();
    launchd.runner.broken.replace(Some("bootout"));
    let failed = launchd.stop("todoist");
    assert_eq!(
        failed,
        Err(ScheduleError::Failed("launchctl bootout: refused".into()))
    );
}
