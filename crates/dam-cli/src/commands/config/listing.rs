use dam_adapters::LuaConfigFile;
use dam_application::ConfigFile;
use serde::Serialize;

use super::setting::Setting;
use super::settings::settings;
use crate::commands::doctor::found;
use crate::error::CliError;
use crate::output::Report;
use crate::style::{column, field, heading};
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct Listing {
    config: Option<String>,
    secrets: Option<String>,
    settings: Setting,
}

impl View for Listing {
    fn human(&self) -> Human {
        let or_none = |path: &Option<String>, name: &str| {
            path.clone().unwrap_or_else(|| format!("no {name}"))
        };
        let files = [
            ("config", or_none(&self.config, "config.lua")),
            ("secrets", or_none(&self.secrets, "secrets.lua")),
        ];
        let settings = self.settings.rows("");
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
}

pub(super) fn run(file: &LuaConfigFile) -> Result<Report, CliError> {
    let config = file.load()?.config;
    report(&Listing {
        config: found(file.config_path()),
        secrets: found(&file.secrets_path()),
        settings: settings(&config),
    })
}
