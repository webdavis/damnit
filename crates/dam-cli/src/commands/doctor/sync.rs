use dam_application::{BackgroundState, LastRun, RemoteConfig, RunEnding, ScheduleError};

use super::helpers::{CONFIG_UNREAD, NO_REMOTES};
use super::machine::Machine;
use crate::args::CheckName;
use crate::commands::check_report::{Check, Outcome, Row};
use crate::commands::remote::ago_to_one_unit_and_never_in_the_future as ago;
use crate::commands::sync::every;

const STALE_AFTER_INTERVALS: u32 = 3;

pub(super) fn check(machine: &Machine) -> Vec<Check> {
    let Ok(config) = machine.config() else {
        return vec![Check::of(CheckName::Sync, Outcome::NotRun, CONFIG_UNREAD)];
    };
    if config.remotes.is_empty() {
        return vec![Check::of(CheckName::Sync, Outcome::NotRun, NO_REMOTES)];
    }
    let rows = config.remotes.iter().map(|remote| row(machine, remote));
    vec![Check::of_rows(CheckName::Sync, rows.collect())]
}

fn row(machine: &Machine, remote: &RemoteConfig) -> Row {
    let name = &remote.name.0;
    if !remote.sync.allowed() {
        return Row::of(name, Outcome::Ok, "not allowed by sync.allowed");
    }
    let seen = machine.background.state(name).map_err(said_by_scheduler);
    let last = machine.run_records.last(name).map_err(said_by_scheduler);
    match (seen, last) {
        (Ok(state), Ok(last)) => said(machine, remote, state, last),
        (Err(why), _) | (_, Err(why)) => {
            let detail = format!("not checked, the scheduler did not answer: {why}");
            Row::of(name, Outcome::NotRun, detail)
        }
    }
}

fn said(
    machine: &Machine,
    remote: &RemoteConfig,
    state: BackgroundState,
    last: Option<LastRun>,
) -> Row {
    let name = &remote.name.0;
    let interval = remote.sync.interval();
    let ran = match &last {
        None => "never run".to_string(),
        Some(run) => format!(
            "last run {}: {}",
            ago(machine.now, run.at),
            ended(&run.ended)
        ),
    };
    let detail = [running(state), format!("every {}", every(interval)), ran]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    let stale_at = machine.now - interval * STALE_AFTER_INTERVALS;
    let stale = last.as_ref().is_none_or(|run| run.at < stale_at);
    let fix = match &last {
        Some(LastRun {
            ended: RunEnding::Failed(_),
            ..
        }) => Some("run"),
        _ if state.at_login && stale && !state.running => Some("start"),
        _ if state.at_login && stale => Some("run"),
        _ => None,
    };
    match fix {
        None => Row::of(name, Outcome::Ok, detail),
        Some(verb) => {
            Row::of(name, Outcome::Issue, detail).advising(format!("run dam sync {verb} {name}"))
        }
    }
}

fn running(state: BackgroundState) -> String {
    match (state.running, state.at_login) {
        (false, false) => "off".into(),
        (now, login) => format!(
            "{}, {}",
            if now { "running" } else { "not running" },
            if login {
                "starts at login"
            } else {
                "not at login"
            }
        ),
    }
}

fn ended(ending: &RunEnding) -> String {
    match ending {
        RunEnding::Synced => "synced".into(),
        RunEnding::NotDue => "not due".into(),
        RunEnding::Failed(why) => format!("failed: {why}"),
    }
}

fn said_by_scheduler(error: ScheduleError) -> String {
    match error {
        ScheduleError::Failed(why) => why,
        ScheduleError::NoScheduler { .. } => "this machine has neither launchd nor systemd".into(),
    }
}
