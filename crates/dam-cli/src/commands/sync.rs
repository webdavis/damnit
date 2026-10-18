use dam_application::{PullReport, PushReport, Repositories, Side, pull, push, resolve};

use crate::args::{PullArgs, PushArgs, ResolveArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::oids::resolve_oid;
use crate::output::Report;

pub(crate) fn run_push(ctx: &mut Context, args: PushArgs) -> Result<Report, CliError> {
    let reports = push(
        Repositories::of(ctx.store.as_ref()),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        &ctx.config,
        args.remote.as_deref(),
    )?;
    Ok(Report {
        human: reports.iter().map(push_line).collect::<Vec<_>>().join("\n"),
        data: serde_json::json!({ "remotes": reports.iter().map(push_json).collect::<Vec<_>>() }),
    })
}

pub(crate) fn run_pull(ctx: &mut Context, args: PullArgs) -> Result<Report, CliError> {
    let reports = pull(
        Repositories::of(ctx.store.as_ref()),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &ctx.config,
        args.remote.as_deref(),
    )?;
    Ok(Report {
        human: reports.iter().map(pull_line).collect::<Vec<_>>().join("\n"),
        data: serde_json::json!({ "remotes": reports.iter().map(pull_json).collect::<Vec<_>>() }),
    })
}

pub(crate) fn run_resolve(ctx: &mut Context, args: ResolveArgs) -> Result<Report, CliError> {
    let oid = resolve_oid(ctx.store.as_ref(), &args.oid)?;
    let side = if args.side.theirs {
        Side::Theirs
    } else {
        Side::Ours
    };
    resolve(
        ctx.store.as_ref(),
        ctx.store.as_ref(),
        ctx.store.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &oid,
        side,
    )?;
    let word = if args.side.theirs { "theirs" } else { "ours" };
    Ok(Report {
        human: format!("{} resolved with {word}", oid.short()),
        data: serde_json::json!({ "oid": oid.to_string(), "side": word }),
    })
}

fn push_line(r: &PushReport) -> String {
    let mut line = format!(
        "{}: {} sent, {} ok, {} failed, {} skipped",
        r.remote.0,
        r.sent,
        r.succeeded,
        r.failed.len(),
        r.skipped
    );
    for (oid, why) in &r.failed {
        line.push_str(&format!("\n  {}: {why}", oid.short()));
    }
    line
}

fn push_json(r: &PushReport) -> serde_json::Value {
    serde_json::json!({
        "remote": r.remote.0,
        "sent": r.sent,
        "succeeded": r.succeeded,
        "skipped": r.skipped,
        "failed": r.failed.iter().map(|(o, w)| serde_json::json!({ "oid": o.to_string(), "why": w })).collect::<Vec<_>>(),
    })
}

fn pull_line(r: &PullReport) -> String {
    format!(
        "{}: {} new, {} updated, {} unchanged, {} conflict(s), {} removed upstream",
        r.remote.0, r.created, r.updated, r.unchanged, r.conflicts, r.removed_upstream
    )
}

fn pull_json(r: &PullReport) -> serde_json::Value {
    serde_json::json!({
        "remote": r.remote.0,
        "created": r.created,
        "updated": r.updated,
        "unchanged": r.unchanged,
        "conflicts": r.conflicts,
        "removed_upstream": r.removed_upstream,
    })
}

#[cfg(test)]
mod tests;
