use std::collections::BTreeSet;

use dam_application::{Added, Dam, UseCaseError};
use dam_domain::ItemHash;

use crate::args::AddArgs;
use crate::context::Context;
use crate::error::CliError;

pub(super) fn added(ctx: &Context, args: &AddArgs) -> Result<(Added, Vec<ItemHash>), CliError> {
    let named = args
        .oids
        .iter()
        .map(|t| ctx.dam().resolve_named(t))
        .collect::<Result<Vec<_>, _>>()?;
    let adding = |dam: &Dam<'_>| match args.all {
        true => dam.add_all(),
        false => dam.add(&named),
    };
    if !args.preview {
        return Ok((adding(&ctx.dam())?, Vec::new()));
    }
    let rehearsed = ctx.dam().rehearse(|dam| {
        let before = waiting(dam)?;
        let added = adding(dam)?;
        let still = waiting(dam)?;
        Ok((added, before.difference(&still).cloned().collect()))
    })?;
    Ok(rehearsed.ran)
}

fn waiting(dam: &Dam<'_>) -> Result<BTreeSet<ItemHash>, UseCaseError> {
    Ok(dam
        .store
        .items()?
        .into_iter()
        .filter(|item| item.is_waiting())
        .map(|item| item.hash)
        .collect())
}
