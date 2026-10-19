mod listing;

use dam_application::{Refusal, RemoteName, Repositories, refresh_stale};

use crate::args::{RemoteArgs, RemoteCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};
use serde::Serialize;

#[derive(Serialize)]
struct RemoteAdded<'a> {
    name: &'a str,
    url: &'a str,
    warnings: Vec<String>,
}

impl View for RemoteAdded<'_> {
    fn human(&self) -> Human {
        let mut lines: Vec<String> = self
            .warnings
            .iter()
            .map(|w| format!("warning: {w}"))
            .collect();
        lines.push(format!("added {} ({})", self.name, self.url));
        Human::Lines(lines)
    }
}

pub(crate) fn run_remote(ctx: &Context, args: RemoteArgs) -> Result<Report, CliError> {
    match args.command {
        RemoteCommand::Add { name, url } => {
            let name = RemoteName::parse(&name).map_err(|e| CliError::Usage(e.to_string()))?;
            if ctx.config.remote(&name.0).is_some() {
                return Err(Refusal::RemoteExists(name.0).into());
            }
            let warning = ctx.config_file.add_remote(&name, &url)?;
            report(&RemoteAdded {
                name: &name.0,
                url: &url,
                warnings: Vec::from_iter(warning),
            })
        }
        RemoteCommand::List => listing::run(ctx),
    }
}

pub(crate) fn refresh_stale_unless_no_pull(ctx: &Context) -> Result<(), CliError> {
    if ctx.no_pull {
        return Ok(());
    }
    Ok(refresh_stale(
        Repositories::of(ctx.store.as_ref()),
        ctx.launcher.as_ref(),
        ctx.credentials.as_ref(),
        ctx.clock.as_ref(),
        ctx.random.as_ref(),
        &ctx.config,
    )?)
}

#[cfg(test)]
mod tests;
