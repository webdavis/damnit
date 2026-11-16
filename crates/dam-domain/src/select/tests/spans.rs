use jiff::Timestamp;

use super::fake::Fake;
use crate::select::DateTest;

fn holds(value: &str, point: &str) -> bool {
    let point: Timestamp = point.parse::<jiff::Zoned>().unwrap().timestamp();
    let test = DateTest::read(value).unwrap_or_else(|| panic!("{value} should read"));
    test.holds(point, &Fake::at("2026-11-20T12:00[UTC]"))
}

#[test]
fn a_day_as_the_end_counts_whole() {
    assert!(holds("..today", "2026-11-20T23:59[UTC]"));
    assert!(!holds("..today", "2026-11-21T00:00[UTC]"));
    assert!(holds("..2026-11-20", "2026-11-20T23:59[UTC]"));
}

#[test]
fn a_period_as_the_end_counts_whole() {
    assert!(holds("2026-10..2026-11", "2026-11-30T23:59[UTC]"));
    assert!(!holds("2026-10..2026-11", "2026-09-30T23:59[UTC]"));
    assert!(holds("..week", "2026-11-22T23:59[UTC]"));
}

#[test]
fn an_instant_as_the_end_is_not_included() {
    assert!(holds("..2026-11-20T09:00", "2026-11-20T08:59[UTC]"));
    assert!(!holds("..2026-11-20T09:00", "2026-11-20T09:00[UTC]"));
    assert!(!holds("now..now+2h", "2026-11-20T14:00[UTC]"));
    assert!(holds("now..now+2h", "2026-11-20T13:59[UTC]"));
}

#[test]
fn the_start_is_included_from_its_first_instant() {
    assert!(holds("today..", "2026-11-20T00:00[UTC]"));
    assert!(!holds("today..", "2026-11-19T23:59[UTC]"));
    assert!(holds("2026-11-20T09:00..", "2026-11-20T09:00[UTC]"));
    assert!(holds("today+1..today+2", "2026-11-22T23:00[UTC]"));
}
