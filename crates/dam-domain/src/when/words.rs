use jiff::Span;
use jiff::civil::Weekday;

use crate::Date;
use crate::weekday::parse_weekday;

pub(super) fn day(text: &str, today: Date) -> Option<Date> {
    let lowered = text.to_ascii_lowercase();
    let words: Vec<&str> = lowered.split_whitespace().collect();
    match words.as_slice() {
        ["today"] => Some(today),
        ["tomorrow"] => today.tomorrow().ok(),
        [name] | ["next", name] => {
            parse_weekday(name).and_then(|day| next_weekday_strictly_after(today, day))
        }
        ["in", count, unit] => ahead(today, count, unit),
        _ => None,
    }
}

pub(super) const ACCEPTED_FORMS: &str =
    "today, tomorrow, a weekday such as mon or monday, next <weekday>, in <n> days|weeks|months";

fn next_weekday_strictly_after(today: Date, day: Weekday) -> Option<Date> {
    today.nth_weekday(1, day).ok()
}

fn ahead(today: Date, count: &str, unit: &str) -> Option<Date> {
    let count: u32 = count.parse().ok()?;
    let span = Span::new();
    let span = match unit {
        "day" | "days" => span.try_days(count),
        "week" | "weeks" => span.try_weeks(count),
        "month" | "months" => span.try_months(count),
        _ => return None,
    }
    .ok()?;
    today.checked_add(span).ok()
}

#[cfg(test)]
mod tests;
