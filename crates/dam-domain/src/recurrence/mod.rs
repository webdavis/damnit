mod error;
mod parse;

use std::num::NonZeroU32;

use jiff::civil::Weekday;
use jiff::{Span, ToSpan};

use crate::Date;

pub use error::RuleError;

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
const MAX_DAYS_SCANNED_BEFORE_GIVING_UP: i32 =
    (WIDEST_WEEKLY_INTERVAL_WITH_WEEKDAYS as i32 + 1) * 7;

impl Rule {
    pub fn next_after(&self, date: Date) -> Option<Date> {
        let candidate = match self.freq {
            Freq::Weekly if !self.by_day.is_empty() => self.next_weekly_on(date),
            Freq::Monthly if !self.by_month_day.is_empty() => self.next_monthly_on(date),
            _ => date.checked_add(self.step()?).ok(),
        }?;
        match self.until {
            Some(until) if candidate > until => None,
            _ => Some(candidate),
        }
    }

    fn step(&self) -> Option<Span> {
        span(self.freq, self.interval.get())
    }

    fn next_weekly_on(&self, from: Date) -> Option<Date> {
        let anchor_week = week_start(from);
        let interval = i64::from(self.interval.get());
        let mut day = from;
        for _ in 0..MAX_DAYS_SCANNED_BEFORE_GIVING_UP {
            day = day.tomorrow().ok()?;
            let weeks_between = i64::from((week_start(day) - anchor_week).get_days() / 7);
            if self.by_day.contains(&day.weekday()) && weeks_between.rem_euclid(interval) == 0 {
                return Some(day);
            }
        }
        None
    }

    fn next_monthly_on(&self, from: Date) -> Option<Date> {
        let step = self.step()?;
        let mut month_start = from.first_of_month();
        for _ in 0..48 {
            let days = if month_start == from.first_of_month() {
                self.by_month_day
                    .iter()
                    .filter(|d| **d > from.day())
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                self.by_month_day.clone()
            };
            for day in days {
                if let Ok(candidate) =
                    jiff::civil::Date::new(month_start.year(), month_start.month(), day)
                {
                    return Some(candidate);
                }
            }
            month_start = month_start.checked_add(step).ok()?;
        }
        None
    }
}

fn week_start(date: Date) -> Date {
    let back = date.weekday().since(Weekday::Monday) as i32;
    date.checked_sub(back.days()).unwrap_or(date)
}

pub fn roll_forward(rule: &Rule, due: Date, completed_on: Date) -> Option<Date> {
    match rule.anchor {
        Anchor::Due => rule.next_after(due),
        Anchor::Completion => rule.next_after(completed_on),
    }
}

#[cfg(test)]
mod tests;
