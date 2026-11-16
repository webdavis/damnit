use dam_application::{Landed, SetChange};
use dam_domain::Path;

use super::{Made, land};
use crate::commands::each::report_preview;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(super) fn run(ctx: &Context, hold: Option<Path>, made: Made) -> Result<Report, CliError> {
    let rehearsed = ctx.dam().rehearse(|dam| {
        if let Some(project) = &hold {
            dam.hold_project(project)?;
        }
        land(dam, made)
    })?;
    let sent = match rehearsed.ran {
        Landed::Sent(sent) => Some(SetChange {
            change: sent.item.change.clone(),
            sent: Some(sent),
        }),
        Landed::Written(_) => None,
    };
    report_preview(ctx, rehearsed.changes, sent)
}
