use super::*;
use crate::calendar_api::{GoogleEvent, Listing};

fn listing() -> Listing {
    Listing {
        calendar: "primary".into(),
        summary: Some("me@example.com".into()),
        time_zone: Some("America/New_York".into()),
        items: vec![],
    }
}

fn google(value: serde_json::Value) -> GoogleEvent {
    serde_json::from_value(value).unwrap()
}

fn live_in(listing: &Listing, value: serde_json::Value) -> (WireObject, i64) {
    match map_event(listing, &google(value)).unwrap() {
        Mapped::Live { object, end } => (*object, end),
        Mapped::Cancelled(id) => panic!("cancelled {id}"),
    }
}

fn live(value: serde_json::Value) -> (WireObject, i64) {
    live_in(&listing(), value)
}

#[test]
fn a_timed_event_maps_every_field_dam_models() {
    let (w, end) = live(serde_json::json!({
        "id": "e1", "status": "tentative", "summary": "Standup", "description": "notes", "location": "Room 4",
        "start": {"dateTime": "2026-09-25T14:00:00-04:00", "timeZone": "America/New_York"},
        "end": {"dateTime": "2026-09-25T14:30:00-04:00", "timeZone": "America/New_York"},
        "transparency": "opaque", "visibility": "private", "eventType": "focusTime", "colorId": "5",
        "organizer": {"email": "boss@example.com", "displayName": "Boss"},
        "attendees": [
            {"email": "me@example.com", "responseStatus": "declined", "self": true},
            {"email": "you@example.com", "responseStatus": "needsAction"},
            {"responseStatus": "accepted"}
        ],
        "conferenceData": {"conferenceSolution": {"name": "Google Meet"}, "entryPoints": [
            {"entryPointType": "phone", "uri": "tel:+1"},
            {"entryPointType": "video", "uri": "https://meet.google.com/abc"}
        ]},
        "attachments": [
            {"fileUrl": "https://drive/x", "title": "Agenda", "mimeType": "application/pdf"},
            {"title": "no url"}
        ]
    }));
    assert_eq!(w.oid, "");
    assert_eq!(w.remote_id.as_deref(), Some("primary/e1"));
    assert_eq!(
        (
            w.kind.as_str(),
            w.subject.as_str(),
            w.body.as_str(),
            w.path.as_str()
        ),
        ("event", "Standup", "notes", "me@example.com/")
    );
    let e = w.event.unwrap();
    assert_eq!(e.start, "2026-09-25T14:00:00-04:00[America/New_York]");
    assert_eq!(e.end, "2026-09-25T14:30:00-04:00[America/New_York]");
    assert_eq!(end, 1_790_361_000);
    assert_eq!(e.timezone.as_deref(), Some("America/New_York"));
    assert_eq!(e.location.as_deref(), Some("Room 4"));
    assert_eq!(
        (
            e.status.as_str(),
            e.transparency.as_str(),
            e.visibility.as_str(),
            e.event_type.as_str()
        ),
        ("tentative", "busy", "private", "focus_time")
    );
    assert_eq!(e.color.as_deref(), Some("5"));
    let organizer = e.organizer.unwrap();
    assert_eq!(
        (organizer.email.as_str(), organizer.name.as_deref()),
        ("boss@example.com", Some("Boss"))
    );
    assert_eq!(
        e.attendees.len(),
        2,
        "an attendee with no email is left out"
    );
    assert!(e.attendees[0].is_self && e.attendees[0].response == "declined");
    assert!(!e.attendees[1].is_self);
    assert_eq!(e.attendees[1].response, "needs_action");
    let c = e.conference.unwrap();
    assert_eq!(
        (c.provider.as_str(), c.url.as_str()),
        ("Google Meet", "https://meet.google.com/abc")
    );
    assert_eq!(
        e.attachments.len(),
        1,
        "an attachment with no url is left out"
    );
    assert_eq!(
        (
            e.attachments[0].url.as_str(),
            e.attachments[0].title.as_str()
        ),
        ("https://drive/x", "Agenda")
    );
    assert_eq!(
        e.attachments[0].mime_type.as_deref(),
        Some("application/pdf")
    );
    assert!(w.labels.is_empty() && w.depends.is_empty() && w.reminders.is_empty());
    assert!(w.recurrence.is_none() && w.task.is_none());
}

#[test]
fn every_google_word_reads_as_dams() {
    let words = |response: &str, visibility: &str, event_type: &str| {
        let (w, _) = live(serde_json::json!({
            "id": "w", "visibility": visibility, "eventType": event_type,
            "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"},
            "attendees": [{"email": "a@x", "responseStatus": response}]
        }));
        let e = w.event.unwrap();
        (e.attendees[0].response.clone(), e.visibility, e.event_type)
    };
    assert_eq!(
        words("accepted", "public", "outOfOffice"),
        ("accepted".into(), "public".into(), "out_of_office".into())
    );
    assert_eq!(
        words("tentative", "confidential", "workingLocation"),
        (
            "tentative".into(),
            "confidential".into(),
            "working_location".into()
        )
    );
    assert_eq!(
        words("declined", "default", "birthday"),
        ("declined".into(), "default".into(), "birthday".into())
    );
}

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
fn absent_and_unknown_words_read_as_googles_defaults() {
    let (w, _) = live(serde_json::json!({
        "id": "e3", "eventType": "fromGmail", "visibility": "somethingNew", "transparency": "transparent",
        "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"},
        "attendees": [{"email": "a@x", "responseStatus": "somethingNew"}]
    }));
    let e = w.event.unwrap();
    assert_eq!(
        (
            e.status.as_str(),
            e.transparency.as_str(),
            e.visibility.as_str(),
            e.event_type.as_str()
        ),
        ("confirmed", "free", "default", "default")
    );
    assert_eq!(e.attendees[0].response, "needs_action");
    assert_eq!(w.subject, "");
    assert_eq!(w.body, "");
}

#[test]
fn a_cancelled_item_is_its_remote_id_whatever_else_it_lacks() {
    let mapped = map_event(
        &listing(),
        &google(serde_json::json!({"id": "gone", "status": "cancelled"})),
    )
    .unwrap();
    assert!(matches!(mapped, Mapped::Cancelled(id) if id == "primary/gone"));
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

#[test]
fn a_calendar_title_is_one_path_segment() {
    let all_day = serde_json::json!({"id": "e", "start": {"date": "2026-09-25"}, "end": {"date": "2026-09-26"}});
    let mut l = listing();
    l.summary = Some("Work/Team".into());
    assert_eq!(live_in(&l, all_day.clone()).0.path, "Work-Team/");
    for nothing in ["  ", ".", ".."] {
        l.summary = Some(nothing.into());
        assert_eq!(
            live_in(&l, all_day.clone()).0.path,
            "primary/",
            "{nothing:?}"
        );
    }
    l.summary = None;
    l.calendar = "team/x@group".into();
    assert_eq!(live_in(&l, all_day).0.path, "team-x@group/");
}
