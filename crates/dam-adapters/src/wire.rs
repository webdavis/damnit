//! The translation between dam's own vocabulary and the helper protocol.
//! Every wire type is named here and nowhere inside the application.

use std::collections::BTreeSet;

use dam_application::{
    IncomingObject, MutationOp, MutationOutcome, PullOutcome, RejectedObject, RemoteCapabilities,
    RemoteMutation,
};
use dam_domain::{
    Attachment, Attendee, Base, Conference, Date, Event, EventStatus, EventType, Field, Kind,
    Object, Oid, Path, Person, Priority, Reminder, ResponseStatus, Task, Transparency, Visibility,
    When,
};
use dam_protocol::{
    Capabilities, Mutation, MutationResult, PullResponse, WireAttachment, WireAttendee,
    WireConference, WireEvent, WireObject, WirePerson, WireTask,
};

/// The oid a pulled object carries until the application resolves the one it
/// tracks that remote id by. A helper sends no oid of its own.
fn placeholder_oid() -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(0))
}

/// What the application understands of a helper's declared capabilities. A
/// kind or field this dam does not know is dropped rather than refused.
pub fn capabilities_from_wire(caps: &Capabilities) -> RemoteCapabilities {
    RemoteCapabilities {
        kinds: caps.kinds.iter().filter_map(|k| Kind::parse(k)).collect(),
        fields: caps.fields.iter().filter_map(|f| Field::parse(f)).collect(),
        credentials: caps.credentials.clone(),
        incremental: caps.incremental,
    }
}

/// Reads one pull response, splitting the objects dam can use from the ones
/// it cannot. An object with no remote id is rejected under an empty name,
/// since there is nothing to track it by.
pub fn pull_from_wire(response: PullResponse) -> PullOutcome {
    let mut objects = Vec::new();
    let mut rejected = Vec::new();
    for mut wire in response.objects {
        let Some(remote_id) = wire.remote_id.take() else {
            rejected.push(RejectedObject {
                remote_id: String::new(),
                why: "the object has no remote_id".into(),
            });
            continue;
        };
        wire.oid = placeholder_oid().to_string();
        match from_wire(&wire) {
            Ok(object) => objects.push(IncomingObject { remote_id, object }),
            Err(why) => rejected.push(RejectedObject {
                remote_id,
                why: why.to_string(),
            }),
        }
    }
    PullOutcome {
        objects,
        rejected,
        removed: response.removed,
        sync: response.sync,
    }
}

pub fn mutation_to_wire(mutation: RemoteMutation) -> Mutation {
    Mutation {
        op: match mutation.op {
            MutationOp::Create => "create",
            MutationOp::Update => "update",
            MutationOp::Delete => "delete",
        }
        .to_string(),
        oid: mutation.oid.to_string(),
        idempotency_key: mutation.idempotency_key,
        remote_id: mutation.remote_id,
        object: mutation.object.as_ref().map(|o| to_wire(o, None)),
        fields: mutation
            .fields
            .iter()
            .map(|f| f.as_str().to_string())
            .collect(),
    }
}

/// A result naming an oid that is not one is dropped: the helper is an
/// untrusted subprocess and there is no change to attribute it to.
pub fn outcomes_from_wire(results: Vec<MutationResult>) -> Vec<MutationOutcome> {
    results
        .into_iter()
        .filter_map(|r| {
            Some(MutationOutcome {
                oid: Oid::parse(&r.oid).ok()?,
                ok: r.ok,
                remote_id: r.remote_id,
                why: r.why,
            })
        })
        .collect()
}

/// The wire word for an object's kind, as `to_wire` writes it.
pub fn wire_kind(object: &Object) -> &'static str {
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

pub fn from_wire(wire: &WireObject) -> Result<Object, String> {
    let base = Base {
        oid: Oid::parse(&wire.oid).map_err(|_| bad("oid", &wire.oid))?,
        subject: wire.subject.clone(),
        body: wire.body.clone(),
        path: Path::parse(&wire.path).map_err(|_| bad("path", &wire.path))?,
        labels: wire.labels.iter().cloned().collect::<BTreeSet<_>>(),
        depends: wire
            .depends
            .iter()
            .map(|d| Oid::parse(d).map_err(|_| bad("depends", d)))
            .collect::<Result<_, _>>()?,
        reminders: wire
            .reminders
            .iter()
            .map(|m| Reminder { minutes_before: *m })
            .collect(),
        recurrence: wire.recurrence.clone(),
    };
    match (wire.kind.as_str(), &wire.task, &wire.event) {
        ("task", Some(t), _) => Ok(Object::Task(Task {
            base,
            done: t.done,
            priority: Priority::new(t.priority)
                .map_err(|_| bad("priority", &t.priority.to_string()))?,
            due: t
                .due
                .as_deref()
                .map(parse_when)
                .transpose()
                .map_err(|v| bad("due", &v))?,
            deadline: t
                .deadline
                .as_deref()
                .map(|d| d.parse::<Date>().map_err(|_| bad("deadline", d)))
                .transpose()?,
            event: t
                .event
                .as_deref()
                .map(|o| Oid::parse(o).map_err(|_| bad("event", o)))
                .transpose()?,
        })),
        ("event", _, Some(e)) => Ok(Object::Event(Event {
            base,
            start: parse_when(&e.start).map_err(|v| bad("start", &v))?,
            end: parse_when(&e.end).map_err(|v| bad("end", &v))?,
            timezone: e.timezone.clone(),
            location: e.location.clone(),
            attendees: e
                .attendees
                .iter()
                .map(|a| {
                    Ok(Attendee {
                        email: a.email.clone(),
                        response: parse_response(&a.response)?,
                    })
                })
                .collect::<Result<_, String>>()?,
            status: match e.status.as_str() {
                "confirmed" => EventStatus::Confirmed,
                "tentative" => EventStatus::Tentative,
                "cancelled" => EventStatus::Cancelled,
                v => return Err(bad("status", v)),
            },
            transparency: match e.transparency.as_str() {
                "busy" => Transparency::Busy,
                "free" => Transparency::Free,
                v => return Err(bad("transparency", v)),
            },
            visibility: match e.visibility.as_str() {
                "default" => Visibility::Default,
                "public" => Visibility::Public,
                "private" => Visibility::Private,
                "confidential" => Visibility::Confidential,
                v => return Err(bad("visibility", v)),
            },
            event_type: match e.event_type.as_str() {
                "default" => EventType::Default,
                "focus_time" => EventType::FocusTime,
                "out_of_office" => EventType::OutOfOffice,
                "working_location" => EventType::WorkingLocation,
                "birthday" => EventType::Birthday,
                v => return Err(bad("event_type", v)),
            },
            color: e.color.clone(),
            organizer: e.organizer.as_ref().map(|p| Person {
                email: p.email.clone(),
                name: p.name.clone(),
            }),
            conference: e.conference.as_ref().map(|c| Conference {
                provider: c.provider.clone(),
                url: c.url.clone(),
            }),
            attachments: e
                .attachments
                .iter()
                .map(|a| Attachment {
                    url: a.url.clone(),
                    title: a.title.clone(),
                    mime_type: a.mime_type.clone(),
                })
                .collect(),
        })),
        (kind, _, _) => Err(bad("kind", kind)),
    }
}

fn when_text(when: &When) -> String {
    match when {
        When::Day(d) => d.to_string(),
        When::At(z) => z.to_string(),
    }
}

fn parse_when(text: &str) -> Result<When, String> {
    if let Ok(z) = text.parse::<jiff::Zoned>() {
        return Ok(When::At(z));
    }
    text.parse::<Date>()
        .map(When::Day)
        .map_err(|_| text.to_string())
}

fn response_text(r: ResponseStatus) -> &'static str {
    match r {
        ResponseStatus::NeedsAction => "needs_action",
        ResponseStatus::Accepted => "accepted",
        ResponseStatus::Declined => "declined",
        ResponseStatus::Tentative => "tentative",
    }
}

fn parse_response(text: &str) -> Result<ResponseStatus, String> {
    Ok(match text {
        "needs_action" => ResponseStatus::NeedsAction,
        "accepted" => ResponseStatus::Accepted,
        "declined" => ResponseStatus::Declined,
        "tentative" => ResponseStatus::Tentative,
        v => return Err(bad("response", v)),
    })
}

fn bad(field: &str, value: &str) -> String {
    format!("{field}: cannot read {value:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_domain::{Event, Object, Oid, Priority, Task, Transparency, When};
    use jiff::civil::date;

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    #[test]
    fn a_task_round_trips_with_a_day_due() {
        let mut t = Task::new(oid(1), "milk");
        t.due = Some(When::Day(date(2026, 9, 25)));
        t.priority = Priority::HIGHEST;
        t.base.labels.insert("errand".into());
        let object = Object::Task(t);
        let wire = to_wire(&object, Some("r1".into()));
        assert_eq!(wire.kind, "task");
        assert_eq!(
            wire.task.as_ref().unwrap().due.as_deref(),
            Some("2026-09-25")
        );
        assert_eq!(wire.task.as_ref().unwrap().priority, 1);
        assert_eq!(from_wire(&wire).unwrap(), object);
    }

    #[test]
    fn an_event_round_trips_with_a_zoned_start() {
        let start = date(2026, 9, 25)
            .at(14, 0, 0, 0)
            .in_tz("America/Denver")
            .unwrap();
        let end = date(2026, 9, 25)
            .at(15, 0, 0, 0)
            .in_tz("America/Denver")
            .unwrap();
        let mut e = Event::new(oid(2), "dentist", When::At(start), When::At(end));
        e.transparency = Transparency::Free;
        let object = Object::Event(e);
        let wire = to_wire(&object, None);
        assert_eq!(wire.kind, "event");
        assert_eq!(
            wire.event.as_ref().unwrap().start,
            "2026-09-25T14:00:00-06:00[America/Denver]"
        );
        assert_eq!(wire.event.as_ref().unwrap().transparency, "free");
        assert_eq!(from_wire(&wire).unwrap(), object);
    }

    #[test]
    fn a_bad_date_names_the_field() {
        let mut wire = to_wire(&Object::Task(Task::new(oid(3), "x")), None);
        wire.task.as_mut().unwrap().due = Some("someday".into());
        assert_eq!(
            from_wire(&wire).unwrap_err(),
            "due: cannot read \"someday\""
        );
    }

    #[test]
    fn an_unknown_kind_is_refused() {
        let mut wire = to_wire(&Object::Task(Task::new(oid(4), "x")), None);
        wire.kind = "note".into();
        assert_eq!(from_wire(&wire).unwrap_err(), "kind: cannot read \"note\"");
    }
}
