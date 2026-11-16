use jiff::Timestamp;

use super::fake::Fake;
use crate::select::DateTest;

const FRIDAY_NOON_PARIS: &str = "2026-11-20T12:00[Europe/Paris]";

fn at(text: &str) -> Timestamp {
    text.parse::<jiff::Zoned>().unwrap().timestamp()
}

fn holds(value: &str, point: &str) -> bool {
    let test = DateTest::read(value).unwrap_or_else(|| panic!("{value} should read"));
    test.holds(at(point), &Fake::at(FRIDAY_NOON_PARIS))
}

#[test]
fn a_day_holds_every_instant_of_that_day_in_the_persons_zone() {
    assert!(holds("2026-11-20", "2026-11-20T00:00[Europe/Paris]"));
    assert!(holds("2026-11-20", "2026-11-20T23:59[Europe/Paris]"));
    assert!(!holds("2026-11-20", "2026-11-21T00:00[Europe/Paris]"));
}

#[test]
fn an_instant_due_tomorrow_in_utc_is_today_in_the_persons_zone() {
    let late = "2026-11-20T23:30[America/Los_Angeles]";
    let los_angeles_friday = Fake::at("2026-11-20T12:00[America/Los_Angeles]");
    let today = DateTest::read("today").unwrap();
    assert!(today.holds(at(late), &los_angeles_friday));
    assert!(!today.holds(at(late), &Fake::at("2026-11-20T12:00[UTC]")));
}

#[test]
fn an_instant_value_holds_its_whole_minute() {
    assert!(holds(
        "2026-11-20T09:00",
        "2026-11-20T09:00:30[Europe/Paris]"
    ));
    assert!(!holds("2026-11-20T09:00", "2026-11-20T09:01[Europe/Paris]"));
}

#[test]
fn periods_cover_a_week_a_month_and_a_year() {
    assert!(holds("2026-W47", "2026-11-16T00:00[Europe/Paris]"));
    assert!(holds("2026-W47", "2026-11-22T23:59[Europe/Paris]"));
    assert!(!holds("2026-W47", "2026-11-23T00:00[Europe/Paris]"));
    assert!(holds("2026-11", "2026-11-30T23:00[Europe/Paris]"));
    assert!(!holds("2026-11", "2026-12-01T00:00[Europe/Paris]"));
    assert!(holds("2026", "2026-01-01T00:00[Europe/Paris]"));
    assert!(!holds("2026", "2025-12-31T23:59[Europe/Paris]"));
}

#[test]
fn anchors_count_in_their_own_unit_and_weeks_start_on_monday() {
    assert!(holds("today", "2026-11-20T08:00[Europe/Paris]"));
    assert!(holds("today+3", "2026-11-23T08:00[Europe/Paris]"));
    assert!(holds("today-1", "2026-11-19T08:00[Europe/Paris]"));
    assert!(holds("week", "2026-11-16T08:00[Europe/Paris]"));
    assert!(!holds("week", "2026-11-23T08:00[Europe/Paris]"));
    assert!(holds("week+1", "2026-11-23T08:00[Europe/Paris]"));
    assert!(holds("month-1", "2026-10-31T08:00[Europe/Paris]"));
    assert!(holds("year+1", "2027-06-01T08:00[Europe/Paris]"));
}

#[test]
fn now_with_a_length_names_an_instant() {
    assert!(holds("now", "2026-11-20T12:00:10[Europe/Paris]"));
    assert!(holds("now+2h", "2026-11-20T14:00[Europe/Paris]"));
    assert!(holds("now-30m", "2026-11-20T11:30[Europe/Paris]"));
}

#[test]
fn every_word_that_sets_one_day_reads_as_a_date() {
    assert!(holds("tomorrow", "2026-11-21T08:00[Europe/Paris]"));
    assert!(holds("mon", "2026-11-23T08:00[Europe/Paris]"));
    assert!(holds("next fri", "2026-11-27T08:00[Europe/Paris]"));
    assert!(holds("in 2 weeks", "2026-12-04T08:00[Europe/Paris]"));
    assert!(holds(
        "2026-11-20T09:00[UTC]",
        "2026-11-20T10:00[Europe/Paris]"
    ));
}

#[test]
fn text_that_names_no_date_is_refused() {
    for value in [
        "someday",
        "",
        "2026-13",
        "2026-W54",
        "today+",
        "week+x",
        "now+",
        "1..2..3",
        "..",
        "a,b",
        "today,tomorrow",
        "nowé",
        "on:",
        "on:0",
        "on:32",
        "on:6tue",
    ] {
        assert!(DateTest::read(value).is_none(), "{value} should be refused");
    }
}
