use std::path::Path;

use dam_adapters::ProjectHookFolder;
use dam_application::{Config, HookFingerprint, ProjectHookSwitches};

use crate::error::CliError;

pub(crate) const CHANGED: &str = "this project's hooks changed since you turned them on, so they did not run; dam hook list shows them, and dam config set project.hooks true turns them on again";

pub(crate) const TOO_LARGE: &str = "this project's .damhooks folder holds more than dam checks, 1000 entries or 8 MiB, so its hooks did not run";

pub(crate) enum ProjectHookAnswer {
    Off,
    On(ProjectHookFolder),
    Changed {
        folder: ProjectHookFolder,
        now: HookFingerprint,
        then: Option<HookFingerprint>,
    },
    TooLarge,
}

impl ProjectHookAnswer {
    pub(crate) fn runs(&self) -> bool {
        matches!(self, ProjectHookAnswer::On(_))
    }

    pub(crate) fn changed(&self) -> bool {
        matches!(self, ProjectHookAnswer::Changed { .. })
    }

    pub(crate) fn held_back(&self) -> Option<&'static str> {
        match self {
            ProjectHookAnswer::Changed { .. } => Some(CHANGED),
            ProjectHookAnswer::TooLarge => Some(TOO_LARGE),
            _ => None,
        }
    }
}

pub(crate) fn answer(
    switches: &dyn ProjectHookSwitches,
    clone: &Path,
    personal: &Config,
) -> Result<ProjectHookAnswer, CliError> {
    let this_clone = switches.switch(clone)?;
    if !personal.project_hooks_on(this_clone.as_ref().map(|switch| switch.on)) {
        return Ok(ProjectHookAnswer::Off);
    }
    let folder = ProjectHookFolder::read(clone)?;
    let Some(now) = folder.fingerprint()? else {
        return Ok(ProjectHookAnswer::TooLarge);
    };
    match this_clone {
        Some(switch) if switch.changed_since_turned_on(&now) => Ok(ProjectHookAnswer::Changed {
            folder,
            now,
            then: switch.fingerprint,
        }),
        _ => Ok(ProjectHookAnswer::On(folder)),
    }
}

#[cfg(test)]
mod tests;
