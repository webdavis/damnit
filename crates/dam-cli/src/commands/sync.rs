use dam_application::{Repositories, Side, pull, push};

use crate::args::{PullArgs, PushArgs, ResolveArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;

mod view;

use view::{PullRow, Pulled, PushRow, Pushed, Resolved};

pub(crate) fn run_push(ctx: &Context, args: PushArgs) -> Result<Report, CliError> {
    let reports = push(
        Repositories::of(&*ctx.store),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        &ctx.config,
        args.remote.as_deref(),
    )?;
    report(&Pushed {
        remotes: reports.iter().map(PushRow::new).collect(),
    })
}

pub(crate) fn run_pull(ctx: &Context, args: PullArgs) -> Result<Report, CliError> {
    let reports = pull(
        Repositories::of(&*ctx.store),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &ctx.config,
        args.remote.as_deref(),
    )?;
    report(&Pulled {
        remotes: reports.iter().map(PullRow::new).collect(),
    })
}

pub(crate) fn run_resolve(ctx: &Context, args: ResolveArgs) -> Result<Report, CliError> {
    let oid = ctx.dam().resolve_object(&args.oid)?;
    let side = if args.side.theirs {
        Side::Theirs
    } else {
        Side::Ours
    };
    ctx.dam().resolve(&oid, side)?;
    let side = if args.side.theirs { "theirs" } else { "ours" };
    report(&Resolved { oid: &oid, side })
}

#[cfg(test)]
mod tests;
