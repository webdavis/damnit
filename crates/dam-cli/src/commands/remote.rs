use crate::context::Context;
use crate::error::CliError;

/// Stub until Task 31 adds the real staleness check and pull.
pub fn maybe_pull_stale(_: &mut Context) -> Result<(), CliError> {
    Ok(())
}
