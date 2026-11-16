mod get;
mod hook_settings;
mod listing;
mod match_function;
mod print_default;
mod project_entry;
mod project_hooks;
mod remote_settings;
pub(crate) mod route;
mod set;
mod setting;
mod settings;
mod token_settings;
mod warnings;

use dam_adapters::LuaConfigFile;
use dam_application::ProjectHookSwitches;
use route::Written;

use crate::args::ConfigCommand;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

pub(crate) fn run(
    file: &LuaConfigFile,
    command: ConfigCommand,
    standing: &Standing,
    switches: &dyn ProjectHookSwitches,
) -> Result<Report, CliError> {
    match command {
        ConfigCommand::List => listing::run(file, standing, switches),
        ConfigCommand::Get { key } => match project_hooks::this_clone(&key, false, standing) {
            Some(clone) => get::this_clone(file, switches, clone),
            None => get::run(file, &key),
        },
        ConfigCommand::Set { key, value, global } => {
            match project_hooks::this_clone(&key, global, standing) {
                Some(clone) => project_hooks::set(switches, clone, Some(&value)),
                None => written(file, &key, Some(&value), global, standing),
            }
        }
        ConfigCommand::Unset { key, global } => {
            match project_hooks::this_clone(&key, global, standing) {
                Some(clone) => project_hooks::set(switches, clone, None),
                None => written(file, &key, None, global, standing),
            }
        }
        ConfigCommand::PrintDefault => print_default::run(),
        ConfigCommand::Validate => Err(CliError::Internal(
            "dam config validate is answered as dam doctor config".into(),
        )),
    }
}

fn written(
    file: &LuaConfigFile,
    key: &str,
    value: Option<&str>,
    global: bool,
    standing: &Standing,
) -> Result<Report, CliError> {
    let verb = match value {
        Some(_) => "config set",
        None => "config unset",
    };
    let written = route::written(key, global, standing, verb)?;
    let named = written.name();
    match written {
        Written::Personal => set::run(file, named, key, value),
        Written::Project(_) if value.is_none() && key == "path" => Err(CliError::Usage(
            "a project needs its path; change it with dam config set path <path>".into(),
        )),
        Written::Project(project) => set::run(&project, named, key, value),
    }
}

#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod project_hooks_tests;
#[cfg(test)]
mod project_write_tests;
#[cfg(test)]
mod set_tests;
