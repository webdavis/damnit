use dam_adapters::STARTER_CONFIG;
use serde::Serialize;

use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct StarterConfig {
    text: &'static str,
}

impl View for StarterConfig {
    fn human(&self) -> Human {
        Human::Lines(vec![self.text.to_string()])
    }
}

pub(super) fn run() -> Result<Report, CliError> {
    report(&StarterConfig {
        text: STARTER_CONFIG,
    })
}
