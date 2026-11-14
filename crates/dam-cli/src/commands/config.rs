use dam_adapters::LuaConfigFile;
use serde::Serialize;

use super::token_warnings::token_warnings;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct ConfigChecked {
    config: String,
    secrets: String,
    warnings: Vec<String>,
}

impl View for ConfigChecked {
    fn human(&self) -> Human {
        Human::Lines(vec![self.config.clone(), self.secrets.clone()])
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

pub(crate) fn check(file: &LuaConfigFile) -> Result<Report, CliError> {
    let config = file.check().map_err(CliError::ConfigCheck)?;
    report(&ConfigChecked {
        config: file.config_path().display().to_string(),
        secrets: file.secrets_path().display().to_string(),
        warnings: token_warnings(&config.tokens).collect(),
    })
}
