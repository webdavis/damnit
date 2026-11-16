use dam_application::{Landed, SetChange, SetChanges};
use dam_domain::Path;

use super::{Made, land};
use crate::commands::each::report_set;
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
    let mut changes: Vec<SetChange> = rehearsed
        .changes
        .into_iter()
        .map(|change| SetChange { change, sent: None })
        .collect();
    if let Landed::Sent(sent) = rehearsed.ran {
        changes.push(SetChange {
            change: sent.item.change.clone(),
            sent: Some(sent),
        });
    }
    let previewed = SetChanges {
        preview: true,
        oids: changes.iter().map(|c| c.change.oid().clone()).collect(),
        changes,
    };
    report_set(ctx, &previewed)
}
