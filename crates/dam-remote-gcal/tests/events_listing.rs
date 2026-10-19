use dam_helper::ServiceError;
use dam_helper::testing;
use dam_remote_gcal::calendar_api::{Calendar, CalendarApi, MAX_PAGES, Window};
use dam_remote_gcal::{ApiError, Endpoints};
use std::collections::HashMap;
use testing::Reply;

fn window() -> Window {
    Window {
        from: "2026-09-15T00:00:00Z".parse().unwrap(),
        to: "2026-12-14T00:00:00Z".parse().unwrap(),
    }
}

fn api(base: &str) -> CalendarApi {
    CalendarApi::new(
        dam_helper::agent(),
        &Endpoints::loopback(base).unwrap(),
        "ya29.ACCESS".into(),
    )
}

fn page(items: serde_json::Value, next: Option<&str>) -> Reply {
    let mut body = serde_json::json!({"summary": "me@example.com", "timeZone": "America/New_York", "items": items});
    if let Some(n) = next {
        body["nextPageToken"] = serde_json::json!(n);
    }
    Reply::json(200, body)
}

#[test]
fn one_calendar_is_asked_for_single_instances_deleted_included_over_the_window() {
    let _guard = testing::guard(
        "one_calendar_is_asked_for_single_instances_deleted_included_over_the_window",
    );
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![page(serde_json::json!([{"id": "e1"}]), None)],
    );
    let google = testing::serve(routes);
    let listing = api(&google.base).events("primary", &window()).unwrap();
    assert_eq!(listing.summary.as_deref(), Some("me@example.com"));
    assert_eq!(listing.time_zone.as_deref(), Some("America/New_York"));
    assert_eq!(listing.items[0].id, "e1");
    let seen = &google.seen()[0];
    assert_eq!(seen.authorization.as_deref(), Some("Bearer ya29.ACCESS"));
    for pair in [
        "singleEvents=true",
        "showDeleted=true",
        "maxResults=2500",
        "timeMin=2026-09-15T00%3A00%3A00Z",
        "timeMax=2026-12-14T00%3A00%3A00Z",
    ] {
        assert!(
            seen.query.contains(pair),
            "{pair} missing from {}",
            seen.query
        );
    }
}

#[test]
fn every_page_is_read_until_google_names_no_next_one() {
    let _guard = testing::guard("every_page_is_read_until_google_names_no_next_one");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![
            page(serde_json::json!([{"id": "e1"}]), Some("p2")),
            page(serde_json::json!([{"id": "e2"}]), None),
        ],
    );
    let google = testing::serve(routes);
    let listing = api(&google.base).events("primary", &window()).unwrap();
    assert_eq!(
        listing
            .items
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>(),
        vec!["e1", "e2"]
    );
    assert!(google.seen()[1].query.contains("pageToken=p2"));
}

#[test]
fn a_calendar_that_pages_forever_is_stopped_at_the_cap() {
    let _guard = testing::guard("a_calendar_that_pages_forever_is_stopped_at_the_cap");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![page(serde_json::json!([]), Some("again"))],
    );
    let google = testing::serve(routes);
    let err = api(&google.base).events("primary", &window()).unwrap_err();
    assert!(
        matches!(err, ApiError::TooManyPages { limit } if limit == MAX_PAGES),
        "{err:?}"
    );
    assert_eq!(google.seen().len(), MAX_PAGES);
}

#[test]
fn a_calendar_id_is_one_percent_encoded_path_segment() {
    let _guard = testing::guard("a_calendar_id_is_one_percent_encoded_path_segment");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/en.usa%23holiday%40group.v.calendar.google.com/events",
        vec![page(serde_json::json!([]), None)],
    );
    let google = testing::serve(routes);
    let listing = api(&google.base)
        .events("en.usa#holiday@group.v.calendar.google.com", &window())
        .unwrap();
    assert_eq!(
        listing.calendar,
        "en.usa#holiday@group.v.calendar.google.com"
    );
}

#[test]
fn a_calendar_google_does_not_know_names_itself_in_the_error() {
    let _guard = testing::guard("a_calendar_google_does_not_know_names_itself_in_the_error");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/nope/events",
        vec![Reply::json(
            404,
            serde_json::json!({"error": {"code": 404, "message": "Not Found"}}),
        )],
    );
    let google = testing::serve(routes);
    let said = api(&google.base)
        .events("nope", &window())
        .unwrap_err()
        .to_string();
    assert!(said.contains("404") && said.contains("Not Found"), "{said}");
}

#[test]
fn a_rate_limit_carries_the_retry_time_google_named() {
    let _guard = testing::guard("a_rate_limit_carries_the_retry_time_google_named");
    let mut routes = HashMap::new();
    routes.insert(
        "GET /calendar/v3/calendars/primary/events",
        vec![Reply::json(429, serde_json::json!({})).with_header("Retry-After", "30")],
    );
    let google = testing::serve(routes);
    let err = api(&google.base).events("primary", &window()).unwrap_err();
    assert!(err.to_string().contains("retry after 30s"), "{err}");
}

#[test]
fn a_huge_error_body_is_bounded_to_one_short_line() {
    let _guard = testing::guard("a_huge_error_body_is_bounded_to_one_short_line");
    let huge = Reply {
        status: 500,
        body: format!("line one\nline two {}", "x".repeat(100 * 1024)),
        headers: vec![],
    };
    let mut routes = HashMap::new();
    routes.insert("GET /calendar/v3/calendars/primary/events", vec![huge]);
    let google = testing::serve(routes);
    let err = api(&google.base).events("primary", &window()).unwrap_err();
    let ApiError::Service(ServiceError::Http {
        status: 500, body, ..
    }) = &err
    else {
        panic!("{err:?}")
    };
    assert_eq!(body.chars().count(), 500);
    assert!(body.starts_with("line one line two x"), "{body:?}");
    assert!(body.ends_with('\u{2026}'), "the cut is marked: {body:?}");
}
