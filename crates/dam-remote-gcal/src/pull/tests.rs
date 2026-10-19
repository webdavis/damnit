use super::*;
use crate::calendar_api::fake::FakeCalendar;

const NOW: &str = "2026-09-22T12:00:00Z";

fn event(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id, "summary": id,
        "start": {"dateTime": "2026-09-23T10:00:00Z"}, "end": {"dateTime": "2026-09-23T11:00:00Z"}
    })
}

fn calendars(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

fn now() -> Timestamp {
    NOW.parse().unwrap()
}

#[test]
fn the_window_is_a_week_back_and_ninety_days_ahead() {
    let w = window(now());
    assert_eq!(w.from.to_string(), "2026-09-15T12:00:00Z");
    assert_eq!(w.to.to_string(), "2026-12-21T12:00:00Z");
}

#[test]
fn every_calendar_is_pulled_and_cancelled_items_come_back_by_id() {
    let google = FakeCalendar::default()
        .listing(
            "primary",
            "me@example.com",
            serde_json::json!([event("a"), {"id": "gone", "status": "cancelled"}]),
        )
        .listing("team@x", "Team", serde_json::json!([event("b")]));
    let response = pull(&google, &calendars(&["primary", "team@x"]), None, now()).unwrap();
    let ids: Vec<&str> = response
        .objects
        .iter()
        .filter_map(|o| o.remote_id.as_deref())
        .collect();
    assert_eq!(ids, vec!["primary/a", "team@x/b"]);
    assert_eq!(response.objects[1].path, "Team/");
    assert_eq!(response.cancelled, vec!["primary/gone"]);
    assert!(response.removed.is_empty());
    assert!(response.sync.is_some());
    let asked = google.windows_asked.borrow();
    assert_eq!(asked[0].from.to_string(), "2026-09-15T12:00:00Z");
    assert_eq!(asked[0].to.to_string(), "2026-12-21T12:00:00Z");
}

#[test]
fn one_unreadable_event_fails_the_whole_pull_and_names_it() {
    let google = FakeCalendar::default().listing(
        "primary",
        "me",
        serde_json::json!([event("a"), {"id": "broken"}]),
    );
    let said = pull(&google, &calendars(&["primary"]), None, now())
        .unwrap_err()
        .to_string();
    assert_eq!(said, "calendar primary, event broken: start is missing");
}

#[test]
fn one_calendar_that_fails_fails_the_pull() {
    let google = FakeCalendar::default().listing("primary", "me", serde_json::json!([event("a")]));
    let err = pull(&google, &calendars(&["primary", "missing"]), None, now()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Google Calendar answered 404: calendar missing not found"
    );
}

#[test]
fn an_event_that_left_the_window_early_comes_back_cancelled_on_the_next_pull() {
    let google = FakeCalendar::default()
        .listing(
            "primary",
            "me",
            serde_json::json!([event("stays"), event("moves")]),
        )
        .listing("primary", "me", serde_json::json!([event("stays")]))
        .listing("primary", "me", serde_json::json!([event("stays")]));
    let primary = calendars(&["primary"]);
    let first = pull(&google, &primary, None, now()).unwrap();
    assert!(first.cancelled.is_empty());
    let second = pull(&google, &primary, first.sync.as_deref(), now()).unwrap();
    assert_eq!(second.cancelled, vec!["primary/moves"]);
    let third = pull(&google, &primary, second.sync.as_deref(), now()).unwrap();
    assert!(third.cancelled.is_empty(), "reported once, then forgotten");
}

#[test]
fn a_calendar_taken_off_the_address_has_its_events_cancelled_once() {
    let google = FakeCalendar::default()
        .listing("primary", "me", serde_json::json!([event("a")]))
        .listing("primary", "me", serde_json::json!([event("a")]))
        .listing("primary", "me", serde_json::json!([event("a")]))
        .listing("team@x", "Team", serde_json::json!([event("b")]));
    let first = pull(&google, &calendars(&["primary", "team@x"]), None, now()).unwrap();
    let primary = calendars(&["primary"]);
    let second = pull(&google, &primary, first.sync.as_deref(), now()).unwrap();
    assert_eq!(
        second.cancelled,
        vec!["team@x/b"],
        "its hours stop reading as busy"
    );
    let third = pull(&google, &primary, second.sync.as_deref(), now()).unwrap();
    assert!(third.cancelled.is_empty(), "reported once, then forgotten");
}
