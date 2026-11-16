use dam_domain::Path;

use crate::args::Command;
use crate::commands::dispatch_a_verb_that_opens_the_store;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::project_scope::Standing;

pub(crate) fn dispatch_in(
    ctx: &Context,
    command: Command,
    standing: Standing,
) -> Result<Report, CliError> {
    crate::gate::admit(ctx.access.as_ref(), &command)?;
    let command = match command {
        Command::New(mut a) if a.parent.is_none() || a.path != Path::root() => {
            a.path = a.path.under(&standing.place(&a.scope)?);
            Command::New(a)
        }
        other => other,
    };
    dispatch_a_verb_that_opens_the_store(ctx, command)
}

#[cfg(test)]
mod tests;
