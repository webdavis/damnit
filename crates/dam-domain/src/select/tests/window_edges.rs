use jiff::{Timestamp, Zoned};

use crate::select::WindowEdge;

fn now() -> Zoned {
    "2026-11-20T12:00[UTC]".parse().unwrap()
}

fn at(text: &str) -> Timestamp {
    text.parse::<Zoned>().unwrap().timestamp()
}

fn opens(text: &str) -> Timestamp {
    WindowEdge::read(text).unwrap().opens(&now()).unwrap()
}

fn closes(text: &str) -> Timestamp {
    WindowEdge::read(text).unwrap().closes(&now()).unwrap()
}

#[test]
fn a_day_or_period_opens_at_its_first_instant() {
    assert_eq!(opens("today"), at("2026-11-20T00:00[UTC]"));
    assert_eq!(opens("week+1"), at("2026-11-23T00:00[UTC]"));
    assert_eq!(opens("2026-12"), at("2026-12-01T00:00[UTC]"));
}

#[test]
fn a_day_or_period_as_the_end_counts_whole() {
    assert_eq!(closes("today"), at("2026-11-21T00:00[UTC]"));
    assert_eq!(closes("today+7"), at("2026-11-28T00:00[UTC]"));
    assert_eq!(closes("2026-W48"), at("2026-11-30T00:00[UTC]"));
    assert_eq!(closes("2026-11-25"), at("2026-11-26T00:00[UTC]"));
}

#[test]
fn an_instant_as_the_end_is_not_included() {
    assert_eq!(closes("now+2h"), at("2026-11-20T14:00[UTC]"));
    assert_eq!(closes("2026-11-25T09:00"), at("2026-11-25T09:00[UTC]"));
    assert_eq!(opens("now"), at("2026-11-20T12:00[UTC]"));
}

#[test]
fn every_date_word_setting_takes_still_reads() {
    assert_eq!(opens("tomorrow"), at("2026-11-21T00:00[UTC]"));
    assert_eq!(closes("tomorrow"), at("2026-11-22T00:00[UTC]"));
}

#[test]
fn a_span_a_repeat_or_none_is_not_an_edge() {
    for text in ["today..week", "on:thu", "none", "not:today", "soon"] {
        assert!(WindowEdge::read(text).is_none(), "{text}");
    }
}
