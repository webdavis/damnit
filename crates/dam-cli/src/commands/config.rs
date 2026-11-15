mod get;
mod hook_settings;
mod listing;
mod match_function;
mod print_default;
mod setting;
mod settings;
mod token_settings;
mod warnings;

use dam_adapters::LuaConfigFile;

use crate::args::ConfigCommand;
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run(file: &LuaConfigFile, command: ConfigCommand) -> Result<Report, CliError> {
    match command {
        ConfigCommand::List => listing::run(file),
        ConfigCommand::Get { key } => get::run(file, &key),
        ConfigCommand::PrintDefault => print_default::run(),
        ConfigCommand::Validate => Err(CliError::Internal(
            "dam config validate is answered as dam doctor config".into(),
        )),
    }
}
