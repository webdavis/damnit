use dam_domain::{Date, When, date_to_set, date_to_set_shapes};

use crate::context::Context;
use crate::error::CliError;

const DATE_WORDS: [&str; 4] = ["tomorrow", "fri", "next mon", "in 3 days"];

pub(crate) fn when_flag(ctx: &Context, flag: &str, text: &str) -> Result<When, CliError> {
    let now = ctx.clock.now().to_zoned(ctx.tz.clone());
    date_to_set(text, ctx.clock.today(), &now).ok_or_else(|| not_one_date(flag, text))
}

pub(crate) fn deadline_flag(ctx: &Context, text: &str) -> Result<Date, CliError> {
    day_flag(ctx, "deadline", text)
}

pub(crate) fn day_flag(ctx: &Context, flag: &str, text: &str) -> Result<Date, CliError> {
    when_flag(ctx, flag, text).map(|w| w.date())
}

fn not_one_date(flag: &str, text: &str) -> CliError {
    CliError::Usage(format!(
        "--{flag} cannot be {text:?}; a date to set is one day or instant: {}, or a word such as {}; run it with --{flag} today",
        date_to_set_shapes().join(", "),
        DATE_WORDS.join(", ")
    ))
}

#[cfg(test)]
mod tests;
