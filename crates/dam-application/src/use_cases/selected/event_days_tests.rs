use dam_domain::{Event, When};
use jiff::civil::date;
use jiff::tz::TimeZone;

use super::event_days::event_days;
use crate::testing::oid;

fn days(start: When, end: When) -> Vec<String> {
    let event = Event::new(oid(1), "e", start, end);
    let days = event_days(&event, &TimeZone::UTC);
    days.iter().map(|d| d.to_string()).collect()
}

fn at(text: &str) -> When {
    When::At(text.parse().unwrap())
}

#[test]
fn an_all_day_event_takes_place_up_to_the_day_before_its_end() {
    let two = days(When::Day(date(2026, 1, 1)), When::Day(date(2026, 1, 3)));
    assert_eq!(two, ["2026-01-01", "2026-01-02"]);
}

#[test]
fn a_timed_event_takes_place_on_each_day_it_touches_in_the_persons_zone() {
    let one = days(at("2026-01-01T09:00[UTC]"), at("2026-01-01T10:00[UTC]"));
    assert_eq!(one, ["2026-01-01"]);
    let night = days(at("2026-01-01T22:00[UTC]"), at("2026-01-02T00:00[UTC]"));
    assert_eq!(night, ["2026-01-01"]);
    let across = days(at("2026-01-01T22:00[UTC]"), at("2026-01-02T01:00[UTC]"));
    assert_eq!(across, ["2026-01-01", "2026-01-02"]);
}
