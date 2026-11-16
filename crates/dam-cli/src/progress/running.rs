use dam_adapters::SystemClock;
use dam_application::Clock;
use dam_domain::Timestamp;

use super::line::{Moment, Stage, line};
use crate::output::Format;

pub(super) struct Running {
    pub(super) name: String,
    pub(super) remote: Option<String>,
    pub(super) doing: String,
    pub(super) did: String,
    pub(super) unit: String,
    pub(super) total: Option<usize>,
    pub(super) done: usize,
    pub(super) started: Timestamp,
    pub(super) drawn: bool,
}

impl Running {
    pub(super) fn age_ms(&self) -> u64 {
        let age = SystemClock.now().duration_since(self.started).as_millis();
        u64::try_from(age.max(0)).unwrap_or(u64::MAX)
    }

    pub(super) fn said(&self, stage: Stage, format: Format) -> String {
        let seconds = SystemClock.now().duration_since(self.started).as_secs();
        let moment = Moment {
            name: &self.name,
            remote: self.remote.as_deref(),
            doing: &self.doing,
            did: &self.did,
            unit: &self.unit,
            done: self.done,
            total: self.total,
            seconds: seconds.max(0).unsigned_abs(),
        };
        line(&moment, stage, format)
    }
}
