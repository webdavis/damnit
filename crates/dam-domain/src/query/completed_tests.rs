use super::*;
use crate::{Categories, Completion, Task};
use jiff::civil::date;
use jiff::tz::{TimeZone, offset};

fn done_at(at: &str) -> Object {
    let mut t = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "x");
    t.completion = Completion::Done {
        at: Some(at.parse().unwrap()),
    };
    Object::Task(t)
}

fn hit(query: &str, object: &Object, zone: &TimeZone) -> bool {
    let expr = parse(query, &Categories::default()).unwrap();
    matches(&expr, object, date(2026, 9, 18), zone)
}

#[test]
fn a_completion_counts_on_the_day_it_was_in_the_owners_zone() {
    let late_evening_in_new_york = done_at("2026-09-19T03:30:00Z");
    let new_york = TimeZone::fixed(offset(-4));
    assert!(hit("completed:today", &late_evening_in_new_york, &new_york));
    assert!(!hit(
        "completed:today",
        &late_evening_in_new_york,
        &TimeZone::UTC
    ));
    assert!(hit(
        "completed:tomorrow",
        &late_evening_in_new_york,
        &TimeZone::UTC
    ));
}

#[test]
fn a_completion_with_no_time_matches_only_none() {
    let mut t = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "x");
    t.completion = Completion::Done { at: None };
    let undated = Object::Task(t);
    assert!(hit("completed:none", &undated, &TimeZone::UTC));
    assert!(!hit("completed:today", &undated, &TimeZone::UTC));
}
