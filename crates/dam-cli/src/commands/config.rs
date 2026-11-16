mod get;
mod hook_settings;
mod listing;
mod match_function;
mod print_default;
mod remote_settings;
mod set;
mod setting;
mod settings;
mod token_settings;
mod warnings;

use dam_adapters::LuaConfigFile;

use crate::args::ConfigCommand;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

pub(crate) fn run(
    file: &LuaConfigFile,
    command: ConfigCommand,
    standing: &Standing,
) -> Result<Report, CliError> {
    match command {
        ConfigCommand::List => listing::run(file, standing),
        ConfigCommand::Get { key } => get::run(file, &key),
        ConfigCommand::Set { key, value } => set::run(file, &key, Some(&value)),
        ConfigCommand::Unset { key } => set::run(file, &key, None),
        ConfigCommand::PrintDefault => print_default::run(),
        ConfigCommand::Validate => Err(CliError::Internal(
            "dam config validate is answered as dam doctor config".into(),
        )),
    }
}

#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod set_tests;
