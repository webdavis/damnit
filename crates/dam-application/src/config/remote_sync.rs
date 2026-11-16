use std::time::Duration;

use super::ConfiguredDuration;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteSync {
    pub interval: Option<ConfiguredDuration>,
    pub allowed: Option<bool>,
}

impl RemoteSync {
    pub const DEFAULT_INTERVAL: &str = "15m";
    pub const FLOOR: Duration = Duration::from_secs(60);
    pub const FLOOR_TEXT: &str = "1m";

    pub fn interval(&self) -> Duration {
        self.interval
            .as_ref()
            .map_or(Duration::from_secs(15 * 60), |i| i.value)
    }

    pub fn allowed(&self) -> bool {
        self.allowed.unwrap_or(true)
    }
}
