use dam_adapters::{LuaConfigFile, OsRandom, SystemClock};
use dam_application::Clock;

use crate::args::Command;
use crate::args::hook::{HookArgs, HookCommand};
use crate::commands::{hook_list, token};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn answer(
    file: &LuaConfigFile,
    command: Command,
) -> Result<Result<Report, Command>, CliError> {
    Ok(Ok(match command {
        Command::Token(args) => token::run(file, &OsRandom, SystemClock.now(), args)?,
        Command::Hook(HookArgs {
            command: Some(HookCommand::List),
        }) => hook_list::run(file)?,
        command => return Ok(Err(command)),
    }))
}
