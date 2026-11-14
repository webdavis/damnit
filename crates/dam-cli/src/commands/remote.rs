mod listing;

use dam_application::{ConfigError, Refusal, RemoteName};

use crate::args::{RemoteArgs, RemoteCommand};
use crate::context::Context;
use crate::error::{CliError, SecretsTightenedText, UseCaseText};
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
        Human::Lines(vec![format!("added {} ({})", self.name, self.url)])
    }

    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

pub(crate) fn run_remote(ctx: &Context, args: RemoteArgs) -> Result<Report, CliError> {
    match args.command {
        RemoteCommand::Add { name, url } => {
            let name = RemoteName::parse(&name).map_err(|e| CliError::Usage(e.to_string()))?;
            let warning = ctx
                .config_file
                .add_remote(&name, &url)
                .map_err(|e| match e {
                    ConfigError::AlreadyHeld { .. } => Refusal::RemoteExists(name.0.clone()).into(),
                    other => CliError::Config(other),
                })?;
            report(&RemoteAdded {
                name: &name.0,
                url: &url,
                warnings: Vec::from_iter(warning.map(|w| SecretsTightenedText(&w).to_string())),
            })
        }
        RemoteCommand::List => listing::run(ctx),
    }
}

pub(crate) fn refresh_stale_unless_no_pull(ctx: &Context) -> Result<(), CliError> {
    if ctx.no_pull {
        return Ok(());
    }
    Ok(ctx
        .dam()
        .refresh_stale(ctx.remotes(), |e| UseCaseText(e).to_string())?)
}

#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod tests;
