mod create;
mod grant_view;

use dam_application::{ConfigFile, Randomness};

use crate::args::{TokenArgs, TokenCommand};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run(
    file: &dyn ConfigFile,
    random: &dyn Randomness,
    args: TokenArgs,
) -> Result<Report, CliError> {
    let config = file.load()?;
    match args.command {
        TokenCommand::Create(a) => create::run(file, &config, random, a),
    }
}
