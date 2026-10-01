use super::*;
use jiff::civil::date;

#[test]
fn a_date_is_its_first_instant_in_the_zone_and_a_zoned_time_is_itself() {
    let zone = jiff::tz::TimeZone::fixed(jiff::tz::offset(-4));
    assert_eq!(
        When::Day(jiff::civil::date(2026, 9, 25))
            .instant(&zone)
            .unwrap()
            .as_second(),
        1_790_308_800
    );
    let at: jiff::Zoned = "2026-09-25T14:00:00-04:00[America/New_York]"
        .parse()
        .unwrap();
    assert_eq!(
        When::At(at.clone()).instant(&jiff::tz::TimeZone::UTC),
        Some(at.timestamp())
    );
}

#[test]
fn a_day_is_all_day_and_its_date_is_itself() {
    let w = When::Day(date(2026, 9, 25));
    assert!(w.is_all_day());
    assert_eq!(w.date(), date(2026, 9, 25));
}

#[test]
fn an_instant_reports_its_local_date() {
    let z = date(2026, 9, 25)
        .at(14, 0, 0, 0)
        .in_tz("America/Denver")
        .unwrap();
    let w = When::At(z);
    assert!(!w.is_all_day());
    assert_eq!(w.date(), date(2026, 9, 25));
}

#[test]
fn parse_human_reads_relative_days_dates_and_local_times() {
    let today = date(2026, 9, 18);
    let tz = jiff::tz::TimeZone::UTC;
    assert_eq!(
        When::parse_human("today", today, &tz).unwrap(),
        When::Day(today)
    );
    assert_eq!(
        When::parse_human("tomorrow", today, &tz).unwrap(),
        When::Day(date(2026, 9, 19))
    );
    assert_eq!(
        When::parse_human("2026-09-25", today, &tz).unwrap(),
        When::Day(date(2026, 9, 25))
    );
    let at = When::parse_human("2026-09-25T14:00", today, &tz).unwrap();
    assert_eq!(at.date(), date(2026, 9, 25));
    assert!(!at.is_all_day());
    assert_eq!(
        When::parse_human("next tuesday", today, &tz).unwrap(),
        When::Day(date(2026, 9, 22))
    );
}

#[test]
fn to_text_round_trips_through_parse_human() {
    let today = date(2026, 9, 18);
    let tz = jiff::tz::TimeZone::UTC;
    for text in ["2026-09-25", "2026-09-25T14:00"] {
        let w = When::parse_human(text, today, &tz).unwrap();
        assert_eq!(When::parse_human(&w.to_text(), today, &tz).unwrap(), w);
    }
}

#[test]
fn parse_human_reads_a_full_zoned_timestamp_as_an_instant() {
    let today = date(2026, 9, 18);
    let tz = jiff::tz::TimeZone::UTC;
    let at = When::parse_human("2026-09-25T09:00:00[America/Denver]", today, &tz).unwrap();
    assert!(!at.is_all_day());
    assert!(matches!(at, When::At(_)));
}

#[test]
fn a_malformed_plain_date_gets_the_friendly_hint() {
    let today = date(2026, 9, 18);
    let tz = jiff::tz::TimeZone::UTC;
    let err = When::parse_human("someday", today, &tz).unwrap_err();
    assert!(err.0.contains("cannot read"), "{}", err.0);
    assert!(err.0.contains("next <weekday>"), "{}", err.0);
}

#[test]
fn a_malformed_local_time_gets_the_friendly_hint() {
    let today = date(2026, 9, 18);
    let tz = jiff::tz::TimeZone::UTC;
    let err = When::parse_human("2026-13-45T09:00", today, &tz).unwrap_err();
    assert!(err.0.contains("cannot read"), "{}", err.0);
}

#[test]
fn a_malformed_zoned_timestamp_gets_the_friendly_hint() {
    let today = date(2026, 9, 18);
    let tz = jiff::tz::TimeZone::UTC;
    let err = When::parse_human("2026-09-25T09:00[Nowhere/Nope]", today, &tz).unwrap_err();
    assert!(err.0.contains("cannot read"), "{}", err.0);
}
