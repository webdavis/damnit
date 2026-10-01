mod loopback;
mod support;

use std::collections::HashMap;

use dam_remote_gcal::Endpoints;
use dam_remote_gcal::calendar_api::CalendarApi;
use dam_remote_gcal::pull::{pull, window};
use loopback::Reply;

const NOW: &str = "2026-09-22T12:00:00Z";

fn api(base: &str) -> CalendarApi {
    CalendarApi::new(
        support::agent(),
        &Endpoints::loopback(base).unwrap(),
        "ya29.A".into(),
    )
}

fn listed(summary: &str, items: serde_json::Value) -> Reply {
    Reply::json(
        200,
        serde_json::json!({"summary": summary, "timeZone": "UTC", "items": items}),
    )
}

fn page(summary: &str, items: serde_json::Value) -> Vec<Reply> {
    vec![listed(summary, items)]
}

fn event(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id, "summary": id,
        "start": {"dateTime": "2026-09-23T10:00:00Z"}, "end": {"dateTime": "2026-09-23T11:00:00Z"}
    })
}

fn calendars(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

#[test]
fn the_window_is_a_week_back_and_ninety_days_ahead() {
    let w = window(NOW.parse().unwrap());
    assert_eq!(w.from.to_string(), "2026-09-15T12:00:00Z");
    assert_eq!(w.to.to_string(), "2026-12-21T12:00:00Z");
}

#[test]
fn every_calendar_is_pulled_and_cancelled_items_come_back_by_id() {
    let _guard = support::guard("every_calendar_is_pulled_and_cancelled_items_come_back_by_id");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        page(
            "me@example.com",
            serde_json::json!([event("a"), {"id": "gone", "status": "cancelled"}]),
        ),
    );
    routes.insert(
        "GET /calendar/v3/calendars/team%40x/events",
        page("Team", serde_json::json!([event("b")])),
    );
    let google = loopback::serve(routes);
    let response = pull(
        &api(&google.base),
        &calendars(&["primary", "team@x"]),
        None,
        NOW.parse().unwrap(),
    )
    .unwrap();
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
    let query = loopback::decoded_fields(&google.seen()[0].query);
    assert_eq!(query["timeMin"], "2026-09-15T12:00:00Z");
    assert_eq!(query["timeMax"], "2026-12-21T12:00:00Z");
}

#[test]
fn one_unreadable_event_fails_the_whole_pull_and_names_it() {
    let _guard = support::guard("one_unreadable_event_fails_the_whole_pull_and_names_it");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        page("me", serde_json::json!([event("a"), {"id": "broken"}])),
    );
    let google = loopback::serve(routes);
    let said = pull(
        &api(&google.base),
        &calendars(&["primary"]),
        None,
        NOW.parse().unwrap(),
    )
    .unwrap_err()
    .to_string();
    assert_eq!(said, "calendar primary, event broken: start is missing");
}

#[test]
fn one_calendar_that_fails_fails_the_pull() {
    let _guard = support::guard("one_calendar_that_fails_fails_the_pull");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        page("me", serde_json::json!([event("a")])),
    );
    let google = loopback::serve(routes);
    let err = pull(
        &api(&google.base),
        &calendars(&["primary", "missing"]),
        None,
        NOW.parse().unwrap(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("404"), "{err}");
}

#[test]
fn an_event_that_left_the_window_early_comes_back_cancelled_on_the_next_pull() {
    let _guard =
        support::guard("an_event_that_left_the_window_early_comes_back_cancelled_on_the_next_pull");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![
            listed("me", serde_json::json!([event("stays"), event("moves")])),
            listed("me", serde_json::json!([event("stays")])),
            listed("me", serde_json::json!([event("stays")])),
        ],
    );
    let google = loopback::serve(routes);
    let now = NOW.parse().unwrap();
    let first = pull(&api(&google.base), &calendars(&["primary"]), None, now).unwrap();
    assert!(first.cancelled.is_empty());
    let second = pull(
        &api(&google.base),
        &calendars(&["primary"]),
        first.sync.as_deref(),
        now,
    )
    .unwrap();
    assert_eq!(second.cancelled, vec!["primary/moves"]);
    let third = pull(
        &api(&google.base),
        &calendars(&["primary"]),
        second.sync.as_deref(),
        now,
    )
    .unwrap();
    assert!(third.cancelled.is_empty(), "reported once, then forgotten");
}

#[test]
fn a_calendar_taken_off_the_address_has_its_events_cancelled_once() {
    let _guard = support::guard("a_calendar_taken_off_the_address_has_its_events_cancelled_once");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![
            listed("me", serde_json::json!([event("a")])),
            listed("me", serde_json::json!([event("a")])),
            listed("me", serde_json::json!([event("a")])),
        ],
    );
    routes.insert(
        "GET /calendar/v3/calendars/team%40x/events",
        page("Team", serde_json::json!([event("b")])),
    );
    let google = loopback::serve(routes);
    let now = NOW.parse().unwrap();
    let first = pull(
        &api(&google.base),
        &calendars(&["primary", "team@x"]),
        None,
        now,
    )
    .unwrap();
    let second = pull(
        &api(&google.base),
        &calendars(&["primary"]),
        first.sync.as_deref(),
        now,
    )
    .unwrap();
    assert_eq!(
        second.cancelled,
        vec!["team@x/b"],
        "its hours stop reading as busy"
    );
    let third = pull(
        &api(&google.base),
        &calendars(&["primary"]),
        second.sync.as_deref(),
        now,
    )
    .unwrap();
    assert!(third.cancelled.is_empty(), "reported once, then forgotten");
}
