mod create;
mod generate;
mod grant_view;
mod list;
mod revoke;

use dam_application::{ConfigFile, Randomness};
use dam_domain::Timestamp;

use crate::args::{Command, TokenArgs, TokenCommand};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run(
    file: &dyn ConfigFile,
    random: &dyn Randomness,
    now: Timestamp,
    args: TokenArgs,
) -> Result<Report, CliError> {
    match args.command {
        TokenCommand::Generate => generate::run(random),
        TokenCommand::Create(a) => create::run(file, &file.load()?, random, a),
        TokenCommand::List => list::run(&file.load()?, now),
        TokenCommand::Revoke { name } => revoke::run(file, &file.load()?, name),
    }
}

pub(crate) fn writes_nothing(command: &Command) -> bool {
    matches!(
        command,
        Command::Token(TokenArgs {
            command: TokenCommand::Generate
        })
    )
}
