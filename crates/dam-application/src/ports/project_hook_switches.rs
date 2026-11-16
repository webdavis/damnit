use std::path::Path;

use super::config_file::ConfigError;
use crate::HookFingerprint;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectHookSwitch {
    pub on: bool,
    pub fingerprint: Option<HookFingerprint>,
}

impl ProjectHookSwitch {
    pub fn on_for(fingerprint: HookFingerprint) -> ProjectHookSwitch {
        ProjectHookSwitch {
            on: true,
            fingerprint: Some(fingerprint),
        }
    }

    pub fn off() -> ProjectHookSwitch {
        ProjectHookSwitch {
            on: false,
            fingerprint: None,
        }
    }

    pub fn changed_since_turned_on(&self, now: &HookFingerprint) -> bool {
        self.on && self.fingerprint.as_ref() != Some(now)
    }
}

pub trait ProjectHookSwitches {
    fn switch(&self, project: &Path) -> Result<Option<ProjectHookSwitch>, ConfigError>;
    fn turn(&self, project: &Path, switch: Option<ProjectHookSwitch>) -> Result<(), ConfigError>;
}

#[cfg(test)]
mod tests;
