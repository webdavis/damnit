use std::cell::RefCell;

use dam_application::{BackgroundJob, BackgroundSync, Change, ScheduleError};

#[derive(Default)]
pub(crate) struct RecordingScheduler {
    pub(crate) calls: RefCell<Vec<String>>,
}

impl RecordingScheduler {
    fn record(&self, call: String) -> Result<Change, ScheduleError> {
        self.calls.borrow_mut().push(call);
        Ok(Change::Now)
    }
}

impl BackgroundSync for RecordingScheduler {
    fn start(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        self.record(format!("start {}", job.arguments().join(" ")))
    }
    fn stop(&self, remote: &str) -> Result<Change, ScheduleError> {
        self.record(format!("stop {remote}"))
    }
    fn enable(&self, job: &BackgroundJob) -> Result<Change, ScheduleError> {
        self.record(format!("enable {}", job.arguments().join(" ")))
    }
    fn disable(&self, remote: &str) -> Result<Change, ScheduleError> {
        self.record(format!("disable {remote}"))
    }
}
