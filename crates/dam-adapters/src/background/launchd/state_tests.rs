use std::time::Duration;

use dam_application::{BackgroundState, BackgroundSync};

use super::Launchd;
use crate::background::testing::{RecordingRunner, job};

#[test]
fn state_says_whether_the_agent_is_loaded_and_whether_it_loads_at_login() {
    let (home, state) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let launchd = Launchd {
        runner: RecordingRunner::default(),
        home: home.path().to_path_buf(),
        state: state.path().to_path_buf(),
    };
    let seen = |running, at_login| Ok(BackgroundState { running, at_login });
    assert_eq!(launchd.state("todoist"), seen(false, false));
    launchd.enable(&job()).unwrap();
    assert_eq!(launchd.state("todoist"), seen(false, true));
    let mut faster = job();
    faster.interval = Some(Duration::from_secs(300));
    launchd.start(&faster).unwrap();
    assert_eq!(launchd.state("todoist"), seen(true, true));
    launchd.runner.broken.replace(Some("print"));
    assert!(launchd.state("todoist").is_err());
}
