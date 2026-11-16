use dam_application::Side;

use crate::args::{PullArgs, PushArgs, ResolveArgs, SyncArgs, SyncCommand};
use crate::commands::hooked::hooked;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

mod pull_view;
mod pulled_hook_view;
mod sync_view;
mod view;

use pull_view::{PullRow, Pulled};
use sync_view::Synced;
use view::{PushRow, Pushed, Resolved};

pub(crate) fn run_push(ctx: &Context, args: PushArgs) -> Result<Report, CliError> {
    let reports = ctx.dam().push(ctx.remotes(), args.remote.as_deref())?;
    report(&Pushed {
        remotes: reports.iter().map(PushRow::new).collect(),
    })
}

pub(crate) fn run_pull(ctx: &Context, args: PullArgs) -> Result<Report, CliError> {
    let reports = ctx.dam().pull(ctx.remotes(), args.remote.as_deref())?;
    report(&Pulled {
        remotes: reports.iter().map(PullRow::new).collect(),
    })
}

pub(crate) fn run_sync(ctx: &Context, args: SyncArgs) -> Result<Report, CliError> {
    let Some(SyncCommand::Run {
        remote,
        when_due,
        interval,
    }) = args.command
    else {
        return Err(CliError::Internal(
            "dam sync alone prints help before the store opens".into(),
        ));
    };
    if !when_due {
        return report(&Synced::new(&ctx.dam().sync_now(ctx.remotes(), &remote)?));
    }
    match ctx.dam().sync_when_due(ctx.remotes(), &remote, interval)? {
        Some(synced) => report(&Synced::new(&synced)),
        None => Ok(Report::nothing()),
    }
}

pub(crate) fn run_resolve(ctx: &Context, args: ResolveArgs) -> Result<Report, CliError> {
    let named = ctx.dam().resolve_named(&args.oid)?;
    let side = if args.side.theirs {
        Side::Theirs
    } else {
        Side::Ours
    };
    let oid = ctx.dam().settle(&named, side)?;
    let side = if args.side.theirs { "theirs" } else { "ours" };
    report(&hooked(ctx, Resolved { oid: &oid, side }))
}

#[cfg(test)]
mod failure_kind_tests;
#[cfg(test)]
mod pull_counts_tests;
#[cfg(test)]
mod pull_text_tests;
#[cfg(test)]
mod sync_view_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
