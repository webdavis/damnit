mod config_check;
mod credentials;
mod database;
mod fix;
mod helpers;
mod machine;
mod pushes;
mod store_reading;
mod sync;

use std::path::Path;

use crate::args::{CheckName, Command, ConfigArgs, ConfigCommand};
use crate::commands::check_report::{self, Check, CheckReport, Heading};
use crate::error::CliError;
use crate::output::Format;
pub(crate) use machine::Machine;

const SETUP: Heading = Heading {
    title: "Setup",
    blurb: "whether this machine is set up to run dam",
};

type Run = fn(&Machine) -> Vec<Check>;

const CHECKS: [(CheckName, Run); 6] = [
    (CheckName::Config, config_check::check),
    (CheckName::Database, database::check),
    (CheckName::Helpers, helpers::check),
    (CheckName::Credentials, credentials::check),
    (CheckName::Pushes, pushes::check),
    (CheckName::Sync, sync::check),
];

pub(crate) struct Asked<'a> {
    names: &'a [CheckName],
    command: &'static str,
    fix: bool,
}

pub(crate) fn asked(command: &Command) -> Option<Asked<'_>> {
    match command {
        Command::Doctor(args) => Some(Asked {
            names: &args.checks,
            command: "doctor",
            fix: args.fix,
        }),
        Command::Config(ConfigArgs {
            command: Some(ConfigCommand::Validate),
        }) => Some(Asked {
            names: &[CheckName::Config],
            command: "config validate",
            fix: false,
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
    let created = match asked.fix {
        true => Some(fix::create_what_is_missing(asked.names, machine)?),
        false => None,
    };
    let checks = CHECKS
        .iter()
        .filter(|(name, _)| asked.names.is_empty() || asked.names.contains(name))
        .flat_map(|(_, check)| check(machine))
        .collect();
    check_report::print(
        CheckReport::new(asked.command, SETUP, checks).creating(created),
        format,
        color,
    )
}

pub(super) fn found(path: &Path) -> Option<String> {
    path.exists().then(|| path.display().to_string())
}
