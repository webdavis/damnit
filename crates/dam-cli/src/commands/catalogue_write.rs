mod entry;
mod filter;
mod filter_written;

use crate::args::FilterCommand;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(super) fn run_filter(ctx: &Context, command: FilterCommand) -> Result<Report, CliError> {
    let file = ctx.settings_file.as_ref();
    match command {
        FilterCommand::Create { name, query } => filter::create(&ctx.config, file, &name, &query),
        FilterCommand::Remove { name } => filter::remove(&ctx.config, file, &name),
        FilterCommand::List => unreachable!("dam filter list reads the config alone"),
    }
}

#[cfg(test)]
mod filter_tests;
