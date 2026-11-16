use jiff::civil::{Date, Weekday};

use super::Context;
use crate::parse_weekday;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OnDay {
    Weekday(Weekday),
    Nth(i8, Weekday),
    DayOfMonth(i8),
    EventsOf(String),
}

impl OnDay {
    pub(crate) fn read(text: &str) -> Option<OnDay> {
        if text.is_empty() {
            return None;
        }
        if let Some(day) = parse_weekday(text) {
            return Some(OnDay::Weekday(day));
        }
        let digits_end = text
            .char_indices()
            .find(|(i, c)| !(c.is_ascii_digit() || (*i == 0 && *c == '-')))
            .map_or(text.len(), |(i, _)| i);
        let (count, name) = text.split_at(digits_end);
        if count.is_empty() {
            return Some(OnDay::EventsOf(text.to_string()));
        }
        let count: i8 = count.parse().ok()?;
        if name.is_empty() {
            return (count != 0 && count.abs() <= 31).then_some(OnDay::DayOfMonth(count));
        }
        let day = parse_weekday(name)?;
        (count != 0 && count.abs() <= 5).then_some(OnDay::Nth(count, day))
    }

    pub(crate) fn holds(&self, day: Date, context: &dyn Context) -> bool {
        let from_start = (day.day() - 1) / 7 + 1;
        let from_end = (day.days_in_month() - day.day()) / 7 + 1;
        match self {
            OnDay::Weekday(w) => day.weekday() == *w,
            OnDay::Nth(n, w) if *n > 0 => day.weekday() == *w && from_start == *n,
            OnDay::Nth(n, w) => day.weekday() == *w && from_end == -n,
            OnDay::DayOfMonth(n) if *n > 0 => day.day() == *n,
            OnDay::DayOfMonth(n) => day.day() == day.days_in_month() + 1 + n,
            OnDay::EventsOf(filter) => context.has_event_on(filter, day),
        }
    }
}
