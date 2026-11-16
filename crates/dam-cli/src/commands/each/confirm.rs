use dam_application::{Selector, SetRun};

use crate::context::Context;
use crate::error::{CliError, Misuse, Problem};

pub(super) fn confirmed(
    ctx: &Context,
    selector: &Selector,
    run: SetRun,
) -> Result<SetRun, CliError> {
    let Some(limit) = ctx.config.confirm_above else {
        return Ok(run);
    };
    let named = matches!(selector, Selector::Oids(_));
    if named || run.preview || run.expect.is_some() || ctx.access.token().is_some() {
        return Ok(run);
    }
    let count = ctx.dam().select(selector)?.len();
    if count <= limit {
        return Ok(run);
    }
    if !ctx.prompt.can_ask() {
        return Err(CliError::CommandLine(Misuse {
            message: format!(
                "{count} objects match, more than confirm_above; add --expect {count} to change them without a terminal"
            ),
            problem: Problem::MissingArgument,
            word: Some("--expect".into()),
        }));
    }
    let answer = ctx.prompt.text(&format!("change {count} objects? [y/N]"))?;
    match answer.to_lowercase().as_str() {
        "y" | "yes" => Ok(SetRun {
            expect: Some(count),
            ..run
        }),
        _ => Err(CliError::Cancelled),
    }
}
