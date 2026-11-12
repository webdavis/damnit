mod create;
mod grant_view;
mod list;
mod revoke;

use dam_application::{ConfigFile, Randomness};
use dam_domain::Timestamp;

use crate::args::{TokenArgs, TokenCommand};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run(
    file: &dyn ConfigFile,
    random: &dyn Randomness,
    now: Timestamp,
    args: TokenArgs,
) -> Result<Report, CliError> {
    let config = file.load()?;
    match args.command {
        TokenCommand::Create(a) => create::run(file, &config, random, a),
        TokenCommand::List => list::run(&config, now),
        TokenCommand::Revoke { name } => revoke::run(file, &config, name),
    }
}
