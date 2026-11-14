use dam_domain::{Date, When};

use crate::context::Context;
use crate::error::CliError;

pub(crate) fn when_flag(ctx: &Context, flag: &str, text: &str) -> Result<When, CliError> {
    When::parse_human(text, ctx.clock.today(), &ctx.tz)
        .map_err(|e| CliError::Usage(format!("--{flag}: {}", e.0)))
}

pub(crate) fn deadline_flag(ctx: &Context, text: &str) -> Result<Date, CliError> {
    day_flag(ctx, "deadline", text)
}

pub(crate) fn day_flag(ctx: &Context, flag: &str, text: &str) -> Result<Date, CliError> {
    When::parse_human(text, ctx.clock.today(), &ctx.tz)
        .map(|w| w.date())
        .map_err(|e| CliError::Usage(format!("--{flag}: {}", e.0)))
}

#[cfg(test)]
mod tests;
