use dam_application::{BackgroundRequest, BackgroundStep, LastRun, RunEnding, Side, UseCaseError};

use crate::args::{PullArgs, PushArgs, ResolveArgs, SyncArgs, SyncCommand};
use crate::commands::hooked::hooked;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::report;
use dam_application::{HelperError, RemoteMutation};
use push_preview_view::PushPreviewView;

mod background_view;
mod failure_reason;
mod pull_view;
mod pulled_hook_view;
mod push_preview_view;
mod sync_view;
mod view;

use background_view::BackgroundSynced;
pub(crate) use background_view::every;
use failure_reason::failure_reason;
use pull_view::{PullRow, Pulled};
use sync_view::Synced;
use view::{PushRow, Pushed, Resolved};

pub(crate) fn run_push(ctx: &Context, args: PushArgs) -> Result<Report, CliError> {
    if args.preview {
        let wire = |m: &RemoteMutation| {
            dam_adapters::declared_on_the_wire(m).map_err(|e| HelperError::Protocol(e.to_string()))
        };
        let previews = ctx.dam().push_preview(args.remote.as_deref(), &wire)?;
        return report(&PushPreviewView::of(&previews));
    }
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
    let background = |step, remote, now, interval| {
        run_background(
            ctx,
            BackgroundRequest {
                remote,
                step,
                now,
                interval,
            },
        )
    };
    match args.command {
        Some(SyncCommand::Run {
            remote,
            when_due,
            interval,
        }) => run_once(ctx, &remote, when_due, interval),
        Some(SyncCommand::Start { remote, interval }) => {
            background(BackgroundStep::Start, remote, false, interval)
        }
        Some(SyncCommand::Stop { remote }) => background(BackgroundStep::Stop, remote, false, None),
        Some(SyncCommand::Enable { remote, now }) => {
            background(BackgroundStep::Enable, remote, now, None)
        }
        Some(SyncCommand::Disable { remote, now }) => {
            background(BackgroundStep::Disable, remote, now, None)
        }
        None => Err(CliError::Internal(
            "dam sync alone prints help before the store opens".into(),
        )),
    }
}

fn run_once(
    ctx: &Context,
    remote: &str,
    when_due: bool,
    interval: Option<std::time::Duration>,
) -> Result<Report, CliError> {
    let synced = match when_due {
        true => ctx.dam().sync_when_due(ctx.remotes(), remote, interval),
        false => ctx.dam().sync_now(ctx.remotes(), remote).map(Some),
    }
    .map_err(CliError::from);
    let ended = match &synced {
        Ok(Some(_)) => RunEnding::Synced,
        Ok(None) => RunEnding::NotDue,
        Err(e) => RunEnding::Failed(failure_reason(e)),
    };
    let at = ctx.clock.now();
    ctx.run_records
        .record(remote, &LastRun { at, ended })
        .map_err(UseCaseError::from)?;
    match synced? {
        Some(synced) => report(&Synced::new(&synced)),
        None => Ok(Report::nothing()),
    }
}

fn run_background(ctx: &Context, request: BackgroundRequest) -> Result<Report, CliError> {
    let done = ctx
        .dam()
        .background_sync(ctx.background.as_ref(), &ctx.invocation, &request)?;
    report(&BackgroundSynced::new(&done))
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
mod background_tests;
#[cfg(test)]
mod background_view_tests;
#[cfg(test)]
mod failure_kind_tests;
#[cfg(test)]
mod pull_counts_tests;
#[cfg(test)]
mod pull_text_tests;
#[cfg(test)]
mod push_preview_tests;
#[cfg(test)]
mod run_record_tests;
#[cfg(test)]
mod sync_view_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
