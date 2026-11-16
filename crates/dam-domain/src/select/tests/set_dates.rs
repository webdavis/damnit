use jiff::Zoned;
use jiff::civil::date;

use crate::When;
use crate::select::{date_to_set, date_to_set_shapes};

fn now() -> Zoned {
    "2026-11-20T09:30[America/New_York]".parse().unwrap()
}

fn set(text: &str) -> Option<When> {
    date_to_set(text, date(2026, 11, 20), &now())
}

#[test]
fn today_with_a_count_of_days_sets_that_day() {
    assert_eq!(set("today+1"), Some(When::Day(date(2026, 11, 21))));
    assert_eq!(set("today-3"), Some(When::Day(date(2026, 11, 17))));
    assert_eq!(set("today"), Some(When::Day(date(2026, 11, 20))));
}

#[test]
fn now_with_a_length_sets_that_instant() {
    let later: Zoned = "2026-11-20T11:30[America/New_York]".parse().unwrap();
    assert_eq!(set("now+2h"), Some(When::At(later)));
    assert_eq!(set("now"), Some(When::At(now())));
}

#[test]
fn every_form_setting_took_before_reads_the_same() {
    let tz = now().time_zone().clone();
    for text in [
        "tomorrow",
        "fri",
        "monday",
        "next mon",
        "in 3 days",
        "in 2 weeks",
        "in 1 month",
        "2026-12-01",
        "2026-12-01T09:00",
        "2026-09-25T09:00:00-04:00",
        "TODAY",
    ] {
        let before = When::parse_human(text, date(2026, 11, 20), &tz).unwrap();
        assert_eq!(set(text), Some(before), "{text}");
    }
}

#[test]
fn a_value_that_names_more_than_one_day_sets_nothing() {
    for text in [
        "2026-11",
        "2026-W47",
        "2026",
        "week",
        "month+1",
        "year",
        "today..fri",
        "..today",
        "on:thu",
        "none",
        "not:today",
        "someday",
    ] {
        assert_eq!(set(text), None, "{text}");
    }
}

#[test]
fn the_shapes_to_set_are_the_selection_shapes_that_name_one_day_or_instant() {
    let shapes = date_to_set_shapes();
    for shape in [
        "2026-11-20",
        "2026-11-20T09:00",
        "today",
        "today+3",
        "now+2h",
    ] {
        assert!(shapes.contains(&shape), "{shape}: {shapes:?}");
    }
    for shape in ["2026-11", "week", "A..B", "on:thu", "not:DATE"] {
        assert!(!shapes.contains(&shape), "{shape}: {shapes:?}");
    }
}
