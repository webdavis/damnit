use std::cell::RefCell;
use std::time::Duration;

use super::*;
use crate::config::RemoteSync;
use crate::errors::Refusal;
use crate::ports::{BackgroundJob, ScheduleError};
use crate::testing::{Scenario, config};

#[derive(Default)]
struct FakeScheduler {
    calls: RefCell<Vec<String>>,
    running: RefCell<bool>,
}

impl FakeScheduler {
    fn record(&self, call: String, now_running: Option<bool>) -> Result<Change, ScheduleError> {
        self.calls.borrow_mut().push(call);
        let Some(wanted) = now_running else {
            return Ok(Change::Now);
        };
        match self.running.replace(wanted) == wanted {
            true => Ok(Change::Already),
            false => Ok(Change::Now),
        }
    }
}

impl BackgroundSync for FakeScheduler {
    fn start(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        let call = format!("start {} {:?}", job.remote, job.interval);
        self.record(call, Some(true))
    }
    fn stop(&self, remote: &str) -> Result<Change, ScheduleError> {
        self.record(format!("stop {remote}"), Some(false))
    }
    fn enable(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        self.record(format!("enable {} {:?}", job.remote, job.interval), None)
    }
    fn disable(&self, remote: &str) -> Result<Change, ScheduleError> {
        self.record(format!("disable {remote}"), None)
    }
}

fn invocation() -> Invocation {
    Invocation {
        dam: "/bin/dam".into(),
        config: "/c/config.lua".into(),
        store: "/s/dam.db".into(),
        search_path: "/bin".into(),
    }
}

fn request(step: BackgroundStep, now: bool, interval: Option<Duration>) -> BackgroundRequest {
    BackgroundRequest {
        remote: "todoist".into(),
        step,
        now,
        interval,
    }
}

fn run(
    s: &Scenario,
    fake: &FakeScheduler,
    r: BackgroundRequest,
) -> Result<BackgroundReport, UseCaseError> {
    s.dam().background_sync(fake, &invocation(), &r)
}

#[test]
fn each_step_makes_one_call_and_now_adds_the_matching_one() {
    let cases = [
        (BackgroundStep::Start, false, vec!["start todoist None"]),
        (BackgroundStep::Stop, false, vec!["stop todoist"]),
        (BackgroundStep::Enable, false, vec!["enable todoist None"]),
        (BackgroundStep::Disable, false, vec!["disable todoist"]),
        (
            BackgroundStep::Enable,
            true,
            vec!["enable todoist None", "start todoist None"],
        ),
        (
            BackgroundStep::Disable,
            true,
            vec!["disable todoist", "stop todoist"],
        ),
    ];
    for (step, now, calls) in cases {
        let fake = FakeScheduler::default();
        run(&Scenario::default(), &fake, request(step, now, None)).unwrap();
        assert_eq!(*fake.calls.borrow(), calls, "{step:?} now={now}");
    }
}

#[test]
fn start_carries_a_temporary_interval_and_reports_it() {
    let fake = FakeScheduler::default();
    let five = Some(Duration::from_secs(300));
    let report = run(
        &Scenario::default(),
        &fake,
        request(BackgroundStep::Start, false, five),
    )
    .unwrap();
    assert_eq!(*fake.calls.borrow(), ["start todoist Some(300s)"]);
    assert_eq!(report.interval, Duration::from_secs(300));
    assert_eq!(report.done, [(BackgroundStep::Start, Change::Now)]);
}

#[test]
fn a_second_start_reports_it_was_already_running() {
    let fake = FakeScheduler::default();
    let s = Scenario::default();
    run(&s, &fake, request(BackgroundStep::Start, false, None)).unwrap();
    let again = run(&s, &fake, request(BackgroundStep::Start, false, None)).unwrap();
    assert_eq!(again.done, [(BackgroundStep::Start, Change::Already)]);
    assert_eq!(again.interval, Duration::from_secs(15 * 60));
}

#[test]
fn sync_not_allowed_refuses_start_and_enable_and_lets_stop_and_disable_through() {
    let mut config = config();
    config.remotes[0].sync = RemoteSync {
        allowed: Some(false),
        ..RemoteSync::default()
    };
    let s = Scenario::default().configured(config);
    let refused = Err(UseCaseError::Refused(Refusal::SyncNotAllowed(
        "todoist".into(),
    )));
    for step in [BackgroundStep::Start, BackgroundStep::Enable] {
        let fake = FakeScheduler::default();
        assert_eq!(run(&s, &fake, request(step, false, None)), refused);
        assert!(fake.calls.borrow().is_empty());
    }
    for step in [BackgroundStep::Stop, BackgroundStep::Disable] {
        assert!(run(&s, &FakeScheduler::default(), request(step, true, None)).is_ok());
    }
}

#[test]
fn an_unknown_remote_is_refused_as_a_pull_refuses_it() {
    let fake = FakeScheduler::default();
    let mut r = request(BackgroundStep::Stop, false, None);
    r.remote = "nope".into();
    let refused = run(&Scenario::default(), &fake, r);
    assert_eq!(
        refused,
        Err(UseCaseError::Refused(Refusal::NoSuchRemote("nope".into())))
    );
    assert!(fake.calls.borrow().is_empty());
}

#[test]
fn a_token_is_refused_before_the_remote_is_looked_up() {
    let s = Scenario::default();
    let token = crate::testing::holder(vec![]);
    let dam = crate::service::Dam {
        access: &token,
        ..s.dam()
    };
    let fake = FakeScheduler::default();
    let mut r = request(BackgroundStep::Start, false, None);
    r.remote = "nope".into();
    let refused = dam.background_sync(&fake, &invocation(), &r);
    let expected = Refusal::NotOpenToTokens("sync start");
    assert_eq!(refused, Err(UseCaseError::Refused(expected)));
    assert!(fake.calls.borrow().is_empty());
}

#[test]
fn a_scheduler_failure_is_reported() {
    struct Broken;
    impl BackgroundSync for Broken {
        fn start(&self, _: &BackgroundJob) -> Result<Change, ScheduleError> {
            Err(ScheduleError::Failed("launchctl bootstrap: boom".into()))
        }
        fn stop(&self, _: &str) -> Result<Change, ScheduleError> {
            unreachable!()
        }
        fn enable(&self, _: &BackgroundJob) -> Result<Change, ScheduleError> {
            unreachable!()
        }
        fn disable(&self, _: &str) -> Result<Change, ScheduleError> {
            unreachable!()
        }
    }
    let r = request(BackgroundStep::Start, false, None);
    let failed = Scenario::default()
        .dam()
        .background_sync(&Broken, &invocation(), &r);
    let expected = ScheduleError::Failed("launchctl bootstrap: boom".into());
    assert_eq!(failed, Err(UseCaseError::Schedule(expected)));
}
