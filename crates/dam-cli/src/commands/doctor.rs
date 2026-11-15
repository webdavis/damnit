mod check;
mod config_check;
mod credentials;
mod database;
mod helpers;
mod machine;
mod pushes;
mod report;
mod row;
mod store_reading;

use std::path::Path;

use crate::args::{CheckName, Command, ConfigArgs, ConfigCommand};
use crate::error::CliError;
use crate::output::{self, Format};
use crate::view;
use check::Check;
pub(crate) use machine::Machine;
use report::Doctor;

type Run = fn(&Machine) -> Vec<Check>;

const CHECKS: [(CheckName, Run); 5] = [
    (CheckName::Config, config_check::check),
    (CheckName::Database, database::check),
    (CheckName::Helpers, helpers::check),
    (CheckName::Credentials, credentials::check),
    (CheckName::Pushes, pushes::check),
];

pub(crate) struct Asked<'a> {
    names: &'a [CheckName],
    command: &'static str,
}

pub(crate) fn asked(command: &Command) -> Option<Asked<'_>> {
    match command {
        Command::Doctor(args) => Some(Asked {
            names: &args.checks,
            command: "doctor",
        }),
        Command::Config(ConfigArgs {
            command: Some(ConfigCommand::Validate),
        }) => Some(Asked {
            names: &[CheckName::Config],
            command: "config validate",
        }),
        _ => None,
    }
}

pub(crate) fn print(
    asked: Asked,
    machine: &Machine,
    format: Format,
    color: bool,
) -> Result<(), CliError> {
    let checks = CHECKS
        .iter()
        .filter(|(name, _)| asked.names.is_empty() || asked.names.contains(name))
        .flat_map(|(_, check)| check(machine))
        .collect();
    let doctor = Doctor::new(asked.command, checks);
    output::print(format, color, &view::report(&doctor)?)?;
    match doctor.found_an_issue() {
        true => Err(CliError::Reported),
        false => Ok(()),
    }
}

pub(super) fn found(path: &Path) -> Option<String> {
    path.exists().then(|| path.display().to_string())
}
