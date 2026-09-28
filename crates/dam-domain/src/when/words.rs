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
mod tests {
    use super::*;
    use jiff::civil::date;

    fn friday() -> Date {
        date(2026, 9, 18)
    }

    #[test]
    fn today_and_tomorrow_read_in_any_case() {
        assert_eq!(day("today", friday()), Some(friday()));
        assert_eq!(day("Today", friday()), Some(friday()));
        assert_eq!(day("TOMORROW", friday()), Some(date(2026, 9, 19)));
    }

    #[test]
    fn a_weekday_is_its_next_occurrence_after_today() {
        assert_eq!(day("mon", friday()), Some(date(2026, 9, 21)));
        assert_eq!(day("monday", friday()), Some(date(2026, 9, 21)));
        assert_eq!(day("Thu", friday()), Some(date(2026, 9, 24)));
        assert_eq!(
            day("fri", friday()),
            Some(date(2026, 9, 25)),
            "today's own weekday means a week out, not today"
        );
    }

    #[test]
    fn next_before_a_weekday_names_the_same_day() {
        for text in ["next mon", "next Monday", "NEXT MON"] {
            assert_eq!(day(text, friday()), Some(date(2026, 9, 21)), "{text}");
        }
        assert_eq!(day("next fri", friday()), day("fri", friday()));
    }

    #[test]
    fn in_a_count_of_days_weeks_or_months_moves_the_date_on() {
        assert_eq!(day("in 3 days", friday()), Some(date(2026, 9, 21)));
        assert_eq!(day("in 1 day", friday()), Some(date(2026, 9, 19)));
        assert_eq!(day("in 1 week", friday()), Some(date(2026, 9, 25)));
        assert_eq!(day("in 2 weeks", friday()), Some(date(2026, 10, 2)));
        assert_eq!(day("in 2 months", friday()), Some(date(2026, 11, 18)));
        assert_eq!(day("in 0 days", friday()), Some(friday()));
    }

    #[test]
    fn a_month_ahead_of_a_day_that_month_lacks_lands_on_its_last() {
        assert_eq!(
            day("in 1 month", date(2026, 1, 31)),
            Some(date(2026, 2, 28))
        );
    }

    #[test]
    fn a_word_outside_the_grammar_is_none_rather_than_a_guess() {
        for text in [
            "",
            "someday",
            "next someday",
            "next",
            "in 3 fortnights",
            "in three days",
            "in -1 days",
            "in 1",
            "2026-09-25",
            "yesterday",
        ] {
            assert_eq!(day(text, friday()), None, "{text}");
        }
    }

    #[test]
    fn a_span_past_the_calendar_is_none_rather_than_a_panic() {
        assert_eq!(day("in 4000000 days", friday()), None);
        assert_eq!(day("in 4000000000 days", friday()), None);
    }
}
