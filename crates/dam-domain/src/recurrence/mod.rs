mod error;
mod next_date;
mod parse;

use std::num::NonZeroU32;

use jiff::Span;
use jiff::civil::Weekday;

use crate::Date;

pub use error::RuleError;
pub use next_date::roll_forward;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Freq {
    pub(crate) fn unit_text(self, plural: bool) -> &'static str {
        match (self, plural) {
            (Freq::Daily, false) => "day",
            (Freq::Daily, true) => "days",
            (Freq::Weekly, false) => "week",
            (Freq::Weekly, true) => "weeks",
            (Freq::Monthly, false) => "month",
            (Freq::Monthly, true) => "months",
            (Freq::Yearly, false) => "year",
            (Freq::Yearly, true) => "years",
        }
    }

    pub(crate) fn widest_steppable_interval(self) -> u32 {
        match self {
            Freq::Daily => 7_304_484,
            Freq::Weekly => 1_043_497,
            Freq::Monthly => 239_976,
            Freq::Yearly => 19_998,
        }
    }
}

fn span(freq: Freq, interval: u32) -> Option<Span> {
    let n = i64::from(interval);
    match freq {
        Freq::Daily => Span::new().try_days(n),
        Freq::Weekly => Span::new().try_weeks(n),
        Freq::Monthly => Span::new().try_months(n),
        Freq::Yearly => Span::new().try_years(n),
    }
    .ok()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Due,
    Completion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    freq: Freq,
    interval: NonZeroU32,
    by_day: Vec<Weekday>,
    by_month_day: Vec<i8>,
    until: Option<Date>,
    anchor: Anchor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recurrence {
    Dam { text: String, rule: Rule },
    Remote(String),
}

impl Recurrence {
    pub fn dam(text: &str) -> Result<Recurrence, RuleError> {
        Rule::parse(text).map(|rule| Recurrence::Dam {
            text: text.to_string(),
            rule,
        })
    }

    pub fn parse(text: &str) -> Recurrence {
        Recurrence::dam(text).unwrap_or_else(|_| Recurrence::Remote(text.to_string()))
    }
}

impl std::fmt::Display for Recurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Recurrence::Dam { text, .. } | Recurrence::Remote(text) => f.write_str(text),
        }
    }
}

pub(crate) const WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS: u32 = 208;

#[cfg(test)]
mod tests;
