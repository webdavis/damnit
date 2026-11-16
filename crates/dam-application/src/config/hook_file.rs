use super::hook::{HookEvent, OnFailure, PulledChanges};
use crate::ports::ConfigError;

#[derive(Debug, PartialEq, Eq)]
pub struct HookFile {
    pub file: String,
    pub event: Option<HookEvent>,
    pub read: Result<HookSetting, ConfigError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookSetting {
    OnFailure(Option<OnFailure>),
    Changes(Option<PulledChanges>),
}
