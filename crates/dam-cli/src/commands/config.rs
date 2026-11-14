use std::path::Path;

use dam_adapters::LuaConfigFile;
use serde::Serialize;

use super::token_warnings::token_warnings;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct ConfigChecked {
    config: Option<String>,
    secrets: Option<String>,
    warnings: Vec<String>,
}

impl View for ConfigChecked {
    fn human(&self) -> Human {
        let or_none = |path: &Option<String>, name: &str| {
            path.clone().unwrap_or_else(|| format!("no {name}"))
        };
        Human::Lines(vec![
            or_none(&self.config, "config.lua"),
            or_none(&self.secrets, "secrets.lua"),
        ])
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

pub(crate) fn check(file: &LuaConfigFile) -> Result<Report, CliError> {
    let config = file.check().map_err(CliError::ConfigCheck)?;
    report(&ConfigChecked {
        config: found(file.config_path()),
        secrets: found(&file.secrets_path()),
        warnings: token_warnings(&config.tokens).collect(),
    })
}

fn found(path: &Path) -> Option<String> {
    path.exists().then(|| path.display().to_string())
}
