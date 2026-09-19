use crate::args::Command;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub fn dispatch(_ctx: &mut Context, command: Command) -> Result<Report, CliError> {
    // Each arm is filled in by the task that adds the verb.
    Err(CliError::Usage(format!(
        "{command:?} is not implemented yet"
    )))
}
