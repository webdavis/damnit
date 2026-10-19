use super::super::*;
use super::{google, listing, live};

#[test]
fn an_all_day_event_is_two_dates_and_ends_at_midnight_in_the_calendars_zone() {
    let (w, end) = live(serde_json::json!({
        "id": "d1", "summary": "Offsite",
        "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}
    }));
    let e = w.event.unwrap();
    assert_eq!(
        (e.start.as_str(), e.end.as_str()),
        ("2026-09-25", "2026-09-26")
    );
    assert_eq!(end, 1_790_395_200, "2026-09-26T00:00-04:00");
    assert_eq!(e.timezone, None);
}

#[test]
fn a_timed_event_with_no_zone_of_its_own_takes_the_calendars() {
    let (w, _) = live(serde_json::json!({
        "id": "e2",
        "start": {"dateTime": "2026-09-25T18:00:00Z"}, "end": {"dateTime": "2026-09-25T19:00:00Z"}
    }));
    assert_eq!(
        w.event.unwrap().start,
        "2026-09-25T14:00:00-04:00[America/New_York]"
    );
}

#[test]
fn a_zone_this_machine_does_not_know_keeps_the_instant_in_utc() {
    let (w, end) = live(serde_json::json!({
        "id": "e4",
        "start": {"dateTime": "2026-09-25T18:00:00Z", "timeZone": "Nowhere/Atlantis"},
        "end": {"dateTime": "2026-09-25T19:00:00Z", "timeZone": "Nowhere/Atlantis"}
    }));
    assert_eq!(w.event.unwrap().start, "2026-09-25T18:00:00+00:00[UTC]");
    assert_eq!(end, 1_790_362_800);
}

#[test]
fn a_live_event_without_a_start_or_an_end_is_an_error_naming_the_calendar_and_the_event() {
    let no_start = map_event(
        &listing(),
        &google(serde_json::json!({"id": "broken", "end": {"date": "2026-09-26"}})),
    )
    .unwrap_err();
    assert_eq!(
        no_start.to_string(),
        "calendar primary, event broken: start is missing"
    );
    let no_end = map_event(
        &listing(),
        &google(serde_json::json!({"id": "broken", "start": {"date": "2026-09-26"}})),
    )
    .unwrap_err();
    assert_eq!(
        no_end.to_string(),
        "calendar primary, event broken: end is missing"
    );
}

#[test]
fn a_time_that_does_not_read_is_an_error() {
    for (time, what) in [
        (
            serde_json::json!({"dateTime": "soon"}),
            "a time is not RFC 3339",
        ),
        (
            serde_json::json!({"date": "2026-13-40"}),
            "a date is not YYYY-MM-DD",
        ),
        (
            serde_json::json!({}),
            "a time names neither date nor dateTime",
        ),
    ] {
        let err = map_event(
            &listing(),
            &google(serde_json::json!({"id": "x", "start": time, "end": {"date": "2026-09-26"}})),
        )
        .unwrap_err();
        assert_eq!(err.what, what);
    }
}
