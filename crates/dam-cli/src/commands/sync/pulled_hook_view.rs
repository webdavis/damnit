use dam_application::PulledHookRun;
use dam_domain::CommitId;
use serde::Serialize;

use crate::style::Mark;
use crate::view::optional_display;

#[derive(Serialize)]
pub(super) struct PulledHookRow {
    ran: usize,
    changed: usize,
    failed: usize,
    not_reached: usize,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "optional_display"
    )]
    commit: Option<CommitId>,
}

impl PulledHookRow {
    pub(super) fn new(run: &PulledHookRun) -> PulledHookRow {
        PulledHookRow {
            ran: run.ran,
            changed: run.changed,
            failed: run.failed,
            not_reached: run.not_reached,
            commit: run.commit.clone(),
        }
    }

    pub(super) fn mark(&self) -> Mark {
        match self.failed {
            0 => Mark::Good,
            _ => Mark::Warn,
        }
    }

    pub(super) fn line(&self) -> String {
        let line = format!(
            "task_pulled: {} ran, {} changed, {} failed, {} not reached",
            self.ran, self.changed, self.failed, self.not_reached
        );
        match &self.commit {
            Some(commit) => format!("{line}, commit {}", commit.short()),
            None => line,
        }
    }
}
