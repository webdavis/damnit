mod view;

use crate::context::Context;
use crate::error::{CliError, UseCaseText};
use crate::last_run;
use crate::output::{self, Format};
use crate::view::report;

use view::{SyncRow, Synced};

pub(crate) fn print(
    ctx: &Context,
    format: Format,
    color: bool,
    record_last_run: bool,
) -> Result<(), CliError> {
    let reports = ctx
        .dam()
        .sync_due_remotes(ctx.remotes(), |e| UseCaseText(e).to_string())?;
    let synced = Synced {
        remotes: reports.iter().map(SyncRow::new).collect(),
    };
    let report = report(&synced)?;
    match record_last_run {
        true => last_run::record_report(format, &report)?,
        false => output::print(format, color, &report)?,
    }
    match synced.any_failed() {
        true => Err(CliError::Reported),
        false => Ok(()),
    }
}
