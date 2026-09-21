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

/// A deadline is a whole day, read through the same words a due date takes;
/// a text carrying a time keeps its date and drops the time.
pub(crate) fn deadline_flag(ctx: &Context, text: &str) -> Result<Date, CliError> {
    When::parse_human(text, ctx.clock.today(), &ctx.tz)
        .map(|w| w.date())
        .map_err(|e| CliError::Usage(format!("--deadline: {}", e.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::context;
    use dam_domain::When;
    use jiff::civil::date;

    /// Every flag that takes a date reads the same words, against the same
    /// clock, so `--due mon` and `--deadline mon` mean one day.
    #[test]
    fn every_date_flag_reads_the_same_words() {
        let ctx = context(); // Friday 2026-09-18
        for flag in ["due", "start", "end"] {
            assert_eq!(
                when_flag(&ctx, flag, "next mon").unwrap(),
                When::Day(date(2026, 9, 21)),
                "--{flag}"
            );
            assert_eq!(
                when_flag(&ctx, flag, "in 2 weeks").unwrap(),
                When::Day(date(2026, 10, 2)),
                "--{flag}"
            );
        }
        assert_eq!(deadline_flag(&ctx, "next mon").unwrap(), date(2026, 9, 21));
        assert_eq!(
            deadline_flag(&ctx, "in 2 weeks").unwrap(),
            date(2026, 10, 2)
        );
        assert_eq!(
            deadline_flag(&ctx, "2026-11-01").unwrap(),
            date(2026, 11, 1)
        );
    }

    /// A deadline in the past is a date like any other: dam keeps no rule
    /// about it, so the flag reads it rather than refusing it.
    #[test]
    fn a_deadline_already_past_is_read_rather_than_refused() {
        let ctx = context();
        assert_eq!(deadline_flag(&ctx, "2020-01-01").unwrap(), date(2020, 1, 1));
    }

    /// An unreadable word is the command line being wrong, not a rule being
    /// broken, so it stays a usage refusal and names what it would accept.
    #[test]
    fn an_unreadable_word_is_a_usage_refusal_naming_the_forms() {
        let ctx = context();
        for error in [
            when_flag(&ctx, "due", "someday").unwrap_err(),
            deadline_flag(&ctx, "someday").unwrap_err(),
        ] {
            assert_eq!(error.exit_code(), 2);
            let said = error.to_string();
            for form in ["today", "next <weekday>", "in <n> days", "YYYY-MM-DD"] {
                assert!(said.contains(form), "{said}");
            }
        }
    }
}
