use jiff::Timestamp;

use super::ScheduleError;

pub trait RunRecords {
    fn record(&self, remote: &str, run: &LastRun) -> Result<(), ScheduleError>;
    fn last(&self, remote: &str) -> Result<Option<LastRun>, ScheduleError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastRun {
    pub at: Timestamp,
    pub ended: RunEnding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEnding {
    Synced,
    NotDue,
    Failed(String),
}
