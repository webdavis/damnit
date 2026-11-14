mod get;
mod listing;
mod print_default;
mod setting;
mod settings;

use dam_adapters::LuaConfigFile;

use super::doctor;
use crate::args::{ConfigArgs, ConfigCommand};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run(file: &LuaConfigFile, args: ConfigArgs) -> Result<Report, CliError> {
    match args.command {
        ConfigCommand::List => listing::run(file),
        ConfigCommand::Get { key } => get::run(file, &key),
        ConfigCommand::PrintDefault => print_default::run(),
        ConfigCommand::Validate => doctor::run(file),
    }
}
