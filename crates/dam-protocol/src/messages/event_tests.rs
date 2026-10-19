use super::*;
use crate::Reply;
use crate::wire::{
    WireAttachment, WireAttendee, WireConference, WireEvent, WireObject, WirePerson,
};

fn standup() -> WireObject {
    WireObject {
        oid: "02".repeat(20),
        remote_id: Some("primary/e1".into()),
        kind: "event".into(),
        subject: "standup".into(),
        body: String::new(),
        path: "me@example.com/".into(),
        labels: vec![],
        depends: vec![],
        reminders: vec![10],
        recurrence: None,
        task: None,
        event: Some(WireEvent {
            start: "2026-09-25T09:00:00-04:00[America/New_York]".into(),
            end: "2026-09-25T09:30:00-04:00[America/New_York]".into(),
            timezone: Some("America/New_York".into()),
            location: Some("Room 4".into()),
            attendees: vec![
                WireAttendee {
                    email: "me@example.com".into(),
                    response: "accepted".into(),
                    is_self: true,
                },
                WireAttendee {
                    email: "ana@example.com".into(),
                    response: "needs_action".into(),
                    is_self: false,
                },
            ],
            status: "confirmed".into(),
            transparency: "busy".into(),
            visibility: "default".into(),
            event_type: "default".into(),
            color: Some("5".into()),
            organizer: Some(WirePerson {
                email: "ana@example.com".into(),
                name: Some("Ana".into()),
            }),
            conference: Some(WireConference {
                provider: "hangoutsMeet".into(),
                url: "https://meet.google.com/abc-defg-hij".into(),
            }),
            attachments: vec![WireAttachment {
                url: "https://drive.google.com/file/d/1".into(),
                title: "agenda".into(),
                mime_type: Some("application/pdf".into()),
            }],
        }),
    }
}

#[test]
fn pull_with_an_event_round_trips_the_fixture() {
    let text = include_str!("../../fixtures/pull-event.response.json").trim_end();
    let answer = Reply::Answered(PullResponse {
        objects: vec![standup()],
        sync: Some("tok-4".into()),
        ..PullResponse::default()
    });
    assert_eq!(serde_json::to_string(&answer).unwrap(), text);
    assert_eq!(
        serde_json::from_str::<Reply<PullResponse>>(text).unwrap(),
        answer
    );
}
