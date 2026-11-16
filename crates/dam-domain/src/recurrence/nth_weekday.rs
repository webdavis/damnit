use jiff::civil::{Date, Weekday};

use super::RuleError;
use crate::weekday::{parse_weekday, weekday_text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NthWeekday {
    nth: i8,
    weekday: Weekday,
}

impl NthWeekday {
    pub(crate) fn parse(text: &str) -> Option<Result<NthWeekday, RuleError>> {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let digits = unsigned.len()
            - unsigned
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .len();
        if digits == 0 {
            return None;
        }
        let Some(weekday) = parse_weekday(&unsigned[digits..]) else {
            return Some(Err(RuleError::BadDay(text.to_string())));
        };
        let sign = text.len() - unsigned.len();
        Some(match text[..sign + digits].parse::<i8>() {
            Ok(nth @ (1..=5 | -5..=-1)) => Ok(NthWeekday { nth, weekday }),
            _ => Err(RuleError::BadOrdinal(text.to_string())),
        })
    }

    pub(crate) fn sort_key(&self) -> (i8, i8) {
        (self.nth, self.weekday.to_monday_zero_offset())
    }

    pub(crate) fn text(&self) -> String {
        format!("{}{}", self.nth, weekday_text(self.weekday))
    }

    pub(crate) fn in_month_of(&self, month_start: Date) -> Option<Date> {
        month_start
            .nth_weekday_of_month(self.nth, self.weekday)
            .ok()
    }
}

#[cfg(test)]
mod tests;
