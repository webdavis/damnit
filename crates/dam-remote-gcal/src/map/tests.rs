mod naming;
mod time;

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
