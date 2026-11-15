use dam_application::PulledHookRun;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct PulledHookRow {
    ran: usize,
    changed: usize,
    failed: usize,
    not_reached: usize,
}

impl PulledHookRow {
    pub(super) fn new(run: &PulledHookRun) -> PulledHookRow {
        PulledHookRow {
            ran: run.ran,
            changed: run.changed,
            failed: run.failed,
            not_reached: run.not_reached,
        }
    }

    pub(super) fn line(&self) -> String {
        format!(
            "task_pulled: {} ran, {} changed, {} failed, {} not reached",
            self.ran, self.changed, self.failed, self.not_reached
        )
    }
}
