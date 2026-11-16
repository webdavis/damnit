use std::time::Duration;

use dam_application::{BackgroundReport, BackgroundStep, Change};
use serde::Serialize;

use crate::view::{Human, View};

#[derive(Serialize)]
pub(super) struct BackgroundSynced<'a> {
    remote: &'a str,
    interval_seconds: u64,
    steps: Vec<StepRow>,
    #[serde(skip)]
    every: String,
}

#[derive(Serialize)]
struct StepRow {
    step: &'static str,
    changed: bool,
    #[serde(skip)]
    kind: BackgroundStep,
}

impl<'a> BackgroundSynced<'a> {
    pub(super) fn new(r: &'a BackgroundReport) -> BackgroundSynced<'a> {
        BackgroundSynced {
            remote: &r.remote,
            interval_seconds: r.interval.as_secs(),
            steps: r.done.iter().map(|(s, c)| row(*s, *c)).collect(),
            every: every(r.interval),
        }
    }

    fn line(&self, row: &StepRow) -> String {
        let every = &self.every;
        let what = match (row.kind, row.changed) {
            (BackgroundStep::Start, true) => {
                format!("syncing in the background every {every} until the next restart")
            }
            (BackgroundStep::Start, false) => {
                format!("already syncing in the background every {every} until the next restart")
            }
            (BackgroundStep::Stop, true) => "background sync stopped".into(),
            (BackgroundStep::Stop, false) => "background sync was not running".into(),
            (BackgroundStep::Enable, true) => {
                format!("background sync starts at every login, every {every}")
            }
            (BackgroundStep::Enable, false) => {
                format!("background sync already starts at every login, every {every}")
            }
            (BackgroundStep::Disable, true) => "background sync no longer starts at login".into(),
            (BackgroundStep::Disable, false) => {
                "background sync already does not start at login".into()
            }
        };
        format!("{}: {what}", self.remote)
    }
}

fn row(kind: BackgroundStep, change: Change) -> StepRow {
    StepRow {
        step: kind.verb().trim_start_matches("sync "),
        changed: change == Change::Now,
        kind,
    }
}

fn every(interval: Duration) -> String {
    match interval.as_secs() {
        s if s % 3600 == 0 => format!("{}h", s / 3600),
        s if s % 60 == 0 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}

impl View for BackgroundSynced<'_> {
    fn human(&self) -> Human {
        Human::Lines(self.steps.iter().map(|r| self.line(r)).collect())
    }
}
