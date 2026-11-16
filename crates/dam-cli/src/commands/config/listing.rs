use dam_adapters::LuaConfigFile;
use dam_application::ConfigFile;
use serde::Serialize;

use super::project_entry::ProjectEntry;
use super::setting::Setting;
use super::settings::settings;
use super::warnings::warnings_naming_only_the_token;
use crate::commands::doctor::found;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;
use crate::style::{column, field, heading};
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct Listing {
    config: Option<String>,
    secrets: Option<String>,
    project: Option<ProjectEntry>,
    settings: Setting,
    warnings: Vec<String>,
}

impl View for Listing {
    fn human(&self) -> Human {
        let or_none = |path: &Option<String>, name: &str| {
            path.clone().unwrap_or_else(|| format!("no {name}"))
        };
        let mut files = vec![
            ("config", or_none(&self.config, "config.lua")),
            ("secrets", or_none(&self.secrets, "secrets.lua")),
        ];
        let mut settings = self.settings.rows("");
        if let Some(project) = &self.project {
            files.push(("project", project.file().to_string()));
            settings.extend(project.rows());
        }
        let files_column = column(files.iter().map(|(name, _)| *name));
        let settings_column = column(settings.iter().map(|(key, _)| key.as_str()));
        Human::styled(|paint| {
            let mut lines = vec![heading(paint, "Files", "where dam reads its configuration")];
            lines.extend(files.iter().map(|(n, v)| field(paint, files_column, n, v)));
            lines.push(String::new());
            lines.push(heading(paint, "Settings", "what dam is using"));
            lines.extend(
                settings
                    .iter()
                    .map(|(k, v)| field(paint, settings_column, k, v)),
            );
            lines
        })
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

pub(super) fn run(file: &LuaConfigFile, standing: &Standing) -> Result<Report, CliError> {
    let config = file.load()?.config;
    let project = match standing {
        Standing::Inside(project) => Some(ProjectEntry::of(project)),
        _ => None,
    };
    let mut warnings = warnings_naming_only_the_token(&config.tokens);
    warnings.extend(standing.warning());
    report(&Listing {
        config: found(file.config_path()),
        secrets: found(&file.secrets_path()),
        project,
        settings: settings(&config),
        warnings,
    })
}
