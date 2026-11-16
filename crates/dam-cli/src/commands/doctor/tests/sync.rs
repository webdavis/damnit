use dam_application::{
    BackgroundJob, BackgroundState, BackgroundSync, Change, LastRun, NoProgress, RunEnding,
    RunRecords, ScheduleError,
};
use jiff::{Timestamp, ToSpan};

use super::{Files, Helpers, rows_of};
use crate::args::CheckName;
use crate::commands::check_report::CheckReport;
use crate::commands::doctor::{Machine, SETUP, checks};
use crate::testing::MemoryRunRecords;

const EVERY_15M: &str =
    "return { remote = { fake = { url = \"fake::\", sync = { interval = \"15m\" } } } }\n";
const NOT_ALLOWED: &str =
    "return { remote = { fake = { url = \"fake::\", sync = { allowed = false } } } }\n";

struct Seen(Result<BackgroundState, ScheduleError>);

impl BackgroundSync for Seen {
    fn start(&self, _: &BackgroundJob) -> Result<Change, ScheduleError> {
        unreachable!()
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
    fn state(&self, _: &str) -> Result<BackgroundState, ScheduleError> {
        self.0.clone()
    }
}

fn now() -> Timestamp {
    Timestamp::from_second(1_800_000_000).unwrap()
}

fn report(
    config: &str,
    running: bool,
    at_login: bool,
    last: Option<(i64, RunEnding)>,
) -> CheckReport {
    let state = Seen(Ok(BackgroundState { running, at_login }));
    report_seeing(config, state, last)
}

fn report_seeing(config: &str, state: Seen, last: Option<(i64, RunEnding)>) -> CheckReport {
    let files = Files::new(config, "return {}\n");
    let records = MemoryRunRecords::default();
    if let Some((minutes, ended)) = last {
        let at = now() - minutes.minutes();
        records.record("fake", &LastRun { at, ended }).unwrap();
    }
    let store = files.store_path();
    let helpers = Helpers::default();
    let mut machine = Machine::new(&files.config, &store, &helpers, &NoProgress)
        .watching(Box::new(state), Box::new(records));
    machine.now = now();
    CheckReport::new("doctor", SETUP, checks(&[CheckName::Sync], &machine))
}

#[test]
fn a_remote_syncing_in_the_background_is_well() {
    let rows = rows_of(&report(EVERY_15M, true, true, Some((4, RunEnding::NotDue))));
    assert_eq!(
        rows,
        [
            "  ✓ sync: 1 remote: 1 ok",
            "    ✓ fake: running, starts at login, every 15m, last run 4m ago: not due"
        ]
    );
}

#[test]
fn a_failed_last_run_is_an_issue() {
    let failed = RunEnding::Failed("helper dam-remote-fake was not found".into());
    let rows = rows_of(&report(EVERY_15M, true, false, Some((2, failed))));
    assert_eq!(
        rows,
        [
            "  ✗ sync: 1 remote: 1 issue",
            "    ✗ fake: running, not at login, every 15m, last run 2m ago: failed: helper dam-remote-fake was not found",
            "      → run dam sync run fake"
        ]
    );
}

#[test]
fn an_enabled_remote_whose_last_run_is_older_than_three_intervals_is_an_issue() {
    let rows = rows_of(&report(
        EVERY_15M,
        false,
        true,
        Some((46, RunEnding::Synced)),
    ));
    assert_eq!(
        rows[1..],
        [
            "    ✗ fake: not running, starts at login, every 15m, last run 46m ago: synced",
            "      → run dam sync start fake"
        ]
    );
    let rows = rows_of(&report(EVERY_15M, true, true, None));
    assert_eq!(
        rows[1..],
        [
            "    ✗ fake: running, starts at login, every 15m, never run",
            "      → run dam sync run fake"
        ]
    );
    let rows = rows_of(&report(
        EVERY_15M,
        true,
        true,
        Some((44, RunEnding::Synced)),
    ));
    assert!(rows[0].starts_with("  ✓"), "{rows:?}");
}

#[test]
fn a_remote_neither_running_nor_enabled_is_off_and_well() {
    let rows = rows_of(&report(EVERY_15M, false, false, None));
    assert_eq!(rows[1], "    ✓ fake: off, every 15m, never run");
}

#[test]
fn a_remote_where_background_sync_is_not_allowed_says_so() {
    let rows = rows_of(&report(NOT_ALLOWED, false, false, None));
    assert_eq!(rows[1], "    ✓ fake: not allowed by sync.allowed");
}

#[test]
fn a_scheduler_that_does_not_answer_leaves_the_remote_unchecked() {
    let broken = Seen(Err(ScheduleError::Failed(
        "launchctl print: refused".into(),
    )));
    let rows = rows_of(&report_seeing(EVERY_15M, broken, None));
    assert_eq!(
        rows[1],
        "    · fake: not checked, the scheduler did not answer: launchctl print: refused"
    );
}

#[test]
fn the_json_rows_name_each_remote() {
    let report = report(EVERY_15M, true, true, Some((4, RunEnding::Synced)));
    let json = serde_json::to_value(&report).unwrap();
    let check = &json["checks"][0];
    assert_eq!(check["name"], "sync");
    assert_eq!(check["rows"][0]["name"], "fake");
    assert_eq!(check["rows"][0]["result"], "ok");
}
