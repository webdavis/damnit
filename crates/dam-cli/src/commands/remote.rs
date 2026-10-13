mod listing;

use dam_adapters::append_remote;
use dam_application::{Notice, Refusal, RemoteName, Repositories, pull};

use crate::args::{RemoteArgs, RemoteCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

pub(crate) fn run_remote(ctx: &mut Context, args: RemoteArgs) -> Result<Report, CliError> {
    match args.command {
        RemoteCommand::Add { name, url } => {
            let name = RemoteName::parse(&name).map_err(|e| CliError::Usage(e.to_string()))?;
            if ctx.config.remote(&name.0).is_some() {
                return Err(Refusal::RemoteExists(name.0).into());
            }
            let warning = append_remote(&ctx.config_path, &name.0, &url)?;
            let mut human = format!("added {} ({url})", name.0);
            if let Some(why) = &warning {
                human.insert_str(0, &format!("warning: {why}\n"));
            }
            Ok(Report {
                human,
                data: serde_json::json!({
                    "name": name.0,
                    "url": url,
                    "warnings": Vec::from_iter(warning),
                }),
            })
        }
        RemoteCommand::List => listing::run(ctx),
    }
}

pub(crate) fn maybe_pull_stale(ctx: &mut Context) -> Result<(), CliError> {
    if ctx.no_pull {
        return Ok(());
    }
    for name in remotes_past_their_stale_window(ctx) {
        pull_leaving_a_notice_rather_than_failing_the_read(ctx, name)?;
    }
    Ok(())
}

fn remotes_past_their_stale_window(ctx: &Context) -> Vec<String> {
    let now = ctx.clock.now();
    ctx.config
        .remotes
        .iter()
        .filter_map(|r| {
            let stale = r.stale?;
            let last = ctx.store.last_pull(&r.name).ok().flatten();
            let age = last.map(|l| now.duration_since(l));
            let is_due = age
                .is_none_or(|a| a.as_secs() >= i64::try_from(stale.as_secs()).unwrap_or(i64::MAX));
            is_due.then(|| r.name.0.clone())
        })
        .collect()
}

fn pull_leaving_a_notice_rather_than_failing_the_read(
    ctx: &Context,
    name: String,
) -> Result<(), CliError> {
    if let Err(e) = pull(
        Repositories::of(ctx.store.as_ref()),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &ctx.config,
        Some(&name),
    ) {
        ctx.store.add_notice(&Notice::PullFailed {
            remote: RemoteName(name),
            why: e.to_string(),
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
