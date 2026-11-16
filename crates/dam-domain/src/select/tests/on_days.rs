use jiff::Timestamp;

use super::fake::Fake;
use crate::select::DateTest;

fn holds(value: &str, day: &str) -> bool {
    holds_with(value, day, &Fake::at("2026-11-20T12:00[UTC]"))
}

fn holds_with(value: &str, day: &str, fake: &Fake) -> bool {
    let point: Timestamp = format!("{day}T10:00Z").parse().unwrap();
    DateTest::read(value).unwrap().holds(point, fake)
}

#[test]
fn a_weekday_holds_every_such_day() {
    assert!(holds("on:thu", "2026-11-19"));
    assert!(holds("on:thursday", "2026-11-26"));
    assert!(!holds("on:thu", "2026-11-20"));
}

#[test]
fn the_nth_weekday_counts_from_the_start_of_the_month() {
    assert!(holds("on:1tue", "2026-12-01"));
    assert!(!holds("on:1tue", "2026-12-08"));
    assert!(holds("on:2tue", "2026-12-08"));
    assert!(holds("on:5tue", "2026-12-29"));
}

#[test]
fn the_last_weekday_counts_from_the_end_of_the_month() {
    assert!(holds("on:-1fri", "2026-11-27"));
    assert!(!holds("on:-1fri", "2026-11-20"));
    assert!(holds("on:-2fri", "2026-11-20"));
    assert!(holds("on:-1sun", "2028-02-27"));
    assert!(holds("on:-1tue", "2028-02-29"));
}

#[test]
fn a_day_of_the_month_counts_from_either_end() {
    assert!(holds("on:15", "2026-11-15"));
    assert!(!holds("on:31", "2026-11-30"));
    assert!(holds("on:-1", "2026-11-30"));
    assert!(holds("on:-1", "2026-12-31"));
    assert!(holds("on:-1", "2028-02-29"));
    assert!(!holds("on:-1", "2028-02-28"));
    assert!(holds("on:-1", "2027-02-28"));
    assert!(holds("on:-2", "2028-02-28"));
}

#[test]
fn a_filter_name_holds_on_the_days_its_events_take_place() {
    let mut fake = Fake::at("2026-11-20T12:00[UTC]");
    fake.event_days
        .entry("allhands".into())
        .or_default()
        .insert("2026-11-24".parse().unwrap());
    assert!(holds_with("on:allhands", "2026-11-24", &fake));
    assert!(!holds_with("on:allhands", "2026-11-25", &fake));
    assert!(!holds_with("on:other", "2026-11-24", &fake));
}
