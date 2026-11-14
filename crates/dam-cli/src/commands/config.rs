mod print_default;

use crate::args::{ConfigArgs, ConfigCommand};
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run(args: ConfigArgs) -> Result<Report, CliError> {
    match args.command {
        ConfigCommand::PrintDefault => print_default::run(),
    }
}
