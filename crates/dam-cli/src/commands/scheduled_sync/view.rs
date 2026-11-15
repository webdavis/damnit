use dam_application::{SyncOutcome, SyncReport};
use serde::Serialize;

use crate::commands::sync::{PullRow, PushRow};
use crate::style::{self, Mark, Paint};
use crate::view::{Human, View};

#[derive(Serialize)]
pub(super) struct Synced<'a> {
    pub(super) remotes: Vec<SyncRow<'a>>,
}

#[derive(Serialize)]
pub(super) struct SyncRow<'a> {
    remote: &'a str,
    sync_every: &'a str,
    #[serde(flatten)]
    result: SyncResult<'a>,
}

#[derive(Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
enum SyncResult<'a> {
    Synced {
        pulled: PullRow<'a>,
        pushed: Option<PushRow<'a>>,
    },
    NotDue,
    Failed {
        why: &'a str,
    },
}

impl<'a> SyncRow<'a> {
    pub(super) fn new(r: &'a SyncReport) -> SyncRow<'a> {
        let result = match &r.outcome {
            SyncOutcome::Synced { pulled, pushed } => SyncResult::Synced {
                pulled: PullRow::new(pulled),
                pushed: pushed.as_ref().map(PushRow::new),
            },
            SyncOutcome::NotDue => SyncResult::NotDue,
            SyncOutcome::Failed { why } => SyncResult::Failed { why },
        };
        SyncRow {
            remote: &r.remote.0,
            sync_every: &r.sync_every.text,
            result,
        }
    }

    fn lines(&self, paint: Paint) -> Vec<String> {
        let remote = self.remote;
        match &self.result {
            SyncResult::Synced { pulled, pushed } => {
                let mut lines = vec![
                    style::row(paint, Mark::Good, 0, &format!("{remote}: synced")),
                    format!("  pulled {}", pulled.counts()),
                ];
                lines.extend(pushed.iter().map(|p| format!("  pushed {}", p.counts())));
                let held = pushed.iter().filter_map(PushRow::held_for_a_conflict);
                lines.extend(held.map(|held| format!("  {held}")));
                lines
            }
            SyncResult::NotDue => {
                let every = self.sync_every;
                let text = format!("{remote}: not due, syncs every {every}");
                vec![style::row(paint, Mark::Quiet, 0, &text)]
            }
            SyncResult::Failed { why } => {
                let text = format!("{remote}: failed: {why}");
                vec![style::row(paint, Mark::Bad, 0, &text)]
            }
        }
    }
}

impl Synced<'_> {
    pub(super) fn any_failed(&self) -> bool {
        self.remotes
            .iter()
            .any(|r| matches!(r.result, SyncResult::Failed { .. }))
    }
}

impl View for Synced<'_> {
    fn human(&self) -> Human {
        Human::styled(|paint| match self.remotes.as_slice() {
            [] => vec![style::row(
                paint,
                Mark::Quiet,
                0,
                "no remote has sync_every",
            )],
            remotes => remotes.iter().flat_map(|r| r.lines(paint)).collect(),
        })
    }
}
