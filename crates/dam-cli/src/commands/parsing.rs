//! Flag parsing shared by `new` and `edit`: a human date, a priority number,
//! and a deadline date, each turned into the matching `Usage` error text.
use dam_domain::{Date, Priority, When};

use crate::context::Context;
use crate::error::CliError;

pub(crate) fn when_flag(ctx: &Context, flag: &str, text: &str) -> Result<When, CliError> {
    When::parse_human(text, ctx.clock.today(), &ctx.tz)
        .map_err(|e| CliError::Usage(format!("--{flag}: {}", e.0)))
}

pub(crate) fn priority_flag(value: u8) -> Result<Priority, CliError> {
    Priority::new(value)
        .map_err(|_| CliError::Usage(format!("-p {value}: priority is 1 (highest) to 4")))
}

pub(crate) fn deadline_flag(text: &str) -> Result<Date, CliError> {
    text.parse::<Date>()
        .map_err(|e| CliError::Usage(format!("--deadline: {e}")))
}
