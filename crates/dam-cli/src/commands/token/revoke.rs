use dam_application::{Config, ConfigFile, Refusal};
use serde::Serialize;

use crate::error::{CliError, SecretsTightenedText};
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct TokenRevoked {
    name: String,
    warnings: Vec<String>,
}

impl View for TokenRevoked {
    fn human(&self) -> Human {
        Human::Lines(vec![format!("revoked token {}", self.name)])
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

pub(super) fn run(
    file: &dyn ConfigFile,
    config: &Config,
    name: String,
) -> Result<Report, CliError> {
    if !config.tokens.names.contains(&name) {
        return Err(Refusal::NoSuchToken(name).into());
    }
    let warning = file.remove_token(&name)?;
    report(&TokenRevoked {
        name,
        warnings: Vec::from_iter(warning.map(|w| SecretsTightenedText(&w).to_string())),
    })
}
