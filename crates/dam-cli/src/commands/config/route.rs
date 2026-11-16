use dam_adapters::{ProjectSettingsFile, personal_setting, project_setting};
use dam_application::{PROJECT_FILE, SettingKey};

use crate::error::{CliError, ConfigReason};
use crate::project_scope::Standing;

pub(crate) const PERSONAL_FILE: &str = "config.lua";

pub(crate) enum Written {
    Personal,
    Project(ProjectSettingsFile),
}

impl Written {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Written::Personal => PERSONAL_FILE,
            Written::Project(_) => PROJECT_FILE,
        }
    }
}

pub(crate) fn written(
    key: &str,
    global: bool,
    standing: &Standing,
    command: &str,
) -> Result<Written, CliError> {
    let Ok(parsed) = SettingKey::parse(key) else {
        return Ok(Written::Personal);
    };
    match (standing, global) {
        (Standing::Inside(project), false) if project_setting(&parsed) => Ok(Written::Project(
            ProjectSettingsFile::new(project.file(), env!("CARGO_PKG_VERSION")),
        )),
        (Standing::Inside(_), false) => Err(CliError::Usage(format!(
            "{key} is yours, not the project's; add --global to change your config.lua"
        ))),
        (Standing::Unreadable(error), false) => Err(CliError::Usage(format!(
            "{}; run dam doctor config, or add --global to change your config.lua",
            ConfigReason(error)
        ))),
        _ if personal_setting(&parsed) => Ok(Written::Personal),
        _ => Err(CliError::Usage(format!(
            "{key} belongs to a project's {PROJECT_FILE}; run dam {command} inside the project's folder"
        ))),
    }
}
