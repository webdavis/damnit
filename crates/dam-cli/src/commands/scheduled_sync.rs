mod view;

use crate::context::Context;
use crate::error::{CliError, UseCaseText};
use crate::output::{self, Format};
use crate::view::report;

use view::{SyncRow, Synced};

pub(crate) fn print(ctx: &Context, format: Format, color: bool) -> Result<(), CliError> {
    let reports = ctx
        .dam()
        .sync_due_remotes(ctx.remotes(), |e| UseCaseText(e).to_string())?;
    let synced = Synced {
        remotes: reports.iter().map(SyncRow::new).collect(),
    };
    output::print(format, color, &report(&synced)?)?;
    match synced.any_failed() {
        true => Err(CliError::Reported),
        false => Ok(()),
    }
}
