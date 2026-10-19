use dam_domain::{Object, When};
use dam_protocol::{
    WireAttachment, WireAttendee, WireConference, WireEvent, WireObject, WirePerson, WireTask,
};

pub fn to_wire(object: &Object, remote_id: Option<String>) -> WireObject {
    let b = object.base();
    WireObject {
        oid: b.oid.to_string(),
        remote_id,
        kind: object.kind().as_str().into(),
        subject: b.subject.clone(),
        body: b.body.clone(),
        path: b.path.to_string(),
        labels: b.labels.iter().cloned().collect(),
        depends: b.depends.iter().map(|o| o.to_string()).collect(),
        reminders: b.reminders.iter().map(|r| r.minutes_before).collect(),
        recurrence: b.recurrence.as_ref().map(ToString::to_string),
        task: object.as_task().map(|t| WireTask {
            done: t.completion.is_done(),
            completed_at: t.completion.at().map(|at| at.to_string()),
            priority: t.priority.get(),
            due: t.due.as_ref().map(When::to_text),
            deadline: t.deadline.map(|d| d.to_string()),
            event: t.event.as_ref().map(|o| o.to_string()),
        }),
        event: object.as_event().map(|e| WireEvent {
            start: e.start.to_text(),
            end: e.end.to_text(),
            timezone: e.timezone.clone(),
            location: e.location.clone(),
            attendees: e
                .attendees
                .iter()
                .map(|a| WireAttendee {
                    email: a.email.clone(),
                    response: a.response.as_str().into(),
                    is_self: a.is_self,
                })
                .collect(),
            status: e.status.as_str().into(),
            transparency: e.transparency.as_str().into(),
            visibility: e.visibility.as_str().into(),
            event_type: e.event_type.as_str().into(),
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
