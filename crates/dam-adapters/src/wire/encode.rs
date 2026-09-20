//! Writing dam's own objects out as the protocol's wire objects.

use dam_domain::{
    EventStatus, EventType, Kind, Object, ResponseStatus, Transparency, Visibility, When,
};
use dam_protocol::{
    WireAttachment, WireAttendee, WireConference, WireEvent, WireObject, WirePerson, WireTask,
};

/// The wire word for an object's kind, as `to_wire` writes it.
fn wire_kind(object: &Object) -> &'static str {
    match object.kind() {
        Kind::Task => "task",
        Kind::Event => "event",
    }
}

pub fn to_wire(object: &Object, remote_id: Option<String>) -> WireObject {
    let b = object.base();
    WireObject {
        oid: b.oid.to_string(),
        remote_id,
        kind: wire_kind(object).into(),
        subject: b.subject.clone(),
        body: b.body.clone(),
        path: b.path.to_string(),
        labels: b.labels.iter().cloned().collect(),
        depends: b.depends.iter().map(|o| o.to_string()).collect(),
        reminders: b.reminders.iter().map(|r| r.minutes_before).collect(),
        recurrence: b.recurrence.clone(),
        task: object.as_task().map(|t| WireTask {
            done: t.done,
            priority: t.priority.get(),
            due: t.due.as_ref().map(when_text),
            deadline: t.deadline.map(|d| d.to_string()),
            event: t.event.as_ref().map(|o| o.to_string()),
        }),
        event: object.as_event().map(|e| WireEvent {
            start: when_text(&e.start),
            end: when_text(&e.end),
            timezone: e.timezone.clone(),
            location: e.location.clone(),
            attendees: e
                .attendees
                .iter()
                .map(|a| WireAttendee {
                    email: a.email.clone(),
                    response: response_text(a.response).into(),
                })
                .collect(),
            status: match e.status {
                EventStatus::Confirmed => "confirmed",
                EventStatus::Tentative => "tentative",
                EventStatus::Cancelled => "cancelled",
            }
            .into(),
            transparency: match e.transparency {
                Transparency::Busy => "busy",
                Transparency::Free => "free",
            }
            .into(),
            visibility: match e.visibility {
                Visibility::Default => "default",
                Visibility::Public => "public",
                Visibility::Private => "private",
                Visibility::Confidential => "confidential",
            }
            .into(),
            event_type: match e.event_type {
                EventType::Default => "default",
                EventType::FocusTime => "focus_time",
                EventType::OutOfOffice => "out_of_office",
                EventType::WorkingLocation => "working_location",
                EventType::Birthday => "birthday",
            }
            .into(),
            color: e.color.clone(),
            organizer: e.organizer.as_ref().map(|p| WirePerson {
                email: p.email.clone(),
                name: p.name.clone(),
            }),
            conference: e.conference.as_ref().map(|c| WireConference {
                provider: c.provider.clone(),
                url: c.url.clone(),
            }),
            attachments: e
                .attachments
                .iter()
                .map(|a| WireAttachment {
                    url: a.url.clone(),
                    title: a.title.clone(),
                    mime_type: a.mime_type.clone(),
                })
                .collect(),
        }),
    }
}

pub(super) fn when_text(when: &When) -> String {
    match when {
        When::Day(d) => d.to_string(),
        When::At(z) => z.to_string(),
    }
}

pub(super) fn response_text(r: ResponseStatus) -> &'static str {
    match r {
        ResponseStatus::NeedsAction => "needs_action",
        ResponseStatus::Accepted => "accepted",
        ResponseStatus::Declined => "declined",
        ResponseStatus::Tentative => "tentative",
    }
}
