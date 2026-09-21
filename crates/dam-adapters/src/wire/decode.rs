//! Reading a protocol wire object back as one of dam's own objects.

use std::collections::BTreeSet;

use dam_domain::{
    Attachment, Attendee, Base, Conference, Date, Event, EventStatus, EventType, Object, Oid,
    OidError, Path, PathError, Person, Priority, PriorityError, Reminder, ResponseStatus, Task,
    Timestamp, Transparency, Visibility, When,
};
use dam_protocol::WireObject;

/// Why one wire object could not be read as one of dam's own. Protocol
/// acceptance over a subprocess's output: a bad oid, an out of range
/// priority, an unknown kind and an unreadable date are four different
/// rejections, and each says which field it was.
#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    Oid {
        field: &'static str,
        value: String,
        why: OidError,
    },
    Path {
        value: String,
        why: PathError,
    },
    Priority {
        value: String,
        why: PriorityError,
    },
    Date {
        field: &'static str,
        value: String,
    },
    UnknownKind(String),
    UnknownEnum {
        field: &'static str,
        value: String,
    },
}

impl WireError {
    /// The field the rejection is about, for counting rejections by class.
    pub fn field(&self) -> &'static str {
        match self {
            WireError::Oid { field, .. } => field,
            WireError::Path { .. } => "path",
            WireError::Priority { .. } => "priority",
            WireError::Date { field, .. } => field,
            WireError::UnknownKind(_) => "kind",
            WireError::UnknownEnum { field, .. } => field,
        }
    }

    fn value(&self) -> &str {
        match self {
            WireError::Oid { value, .. }
            | WireError::Path { value, .. }
            | WireError::Priority { value, .. }
            | WireError::Date { value, .. }
            | WireError::UnknownKind(value)
            | WireError::UnknownEnum { value, .. } => value,
        }
    }
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: cannot read {:?}", self.field(), self.value())
    }
}

impl std::error::Error for WireError {}

fn oid<'a>(field: &'static str, text: &'a str) -> impl Fn(OidError) -> WireError + 'a {
    move |why| WireError::Oid {
        field,
        value: text.to_string(),
        why,
    }
}

fn date(field: &'static str) -> impl Fn(String) -> WireError {
    move |value| WireError::Date { field, value }
}

fn unknown(field: &'static str) -> impl Fn(&str) -> WireError {
    move |value| WireError::UnknownEnum {
        field,
        value: value.to_string(),
    }
}

pub fn from_wire(wire: &WireObject) -> Result<Object, WireError> {
    let base = Base {
        oid: Oid::parse(&wire.oid).map_err(oid("oid", &wire.oid))?,
        subject: wire.subject.clone(),
        body: wire.body.clone(),
        path: Path::parse(&wire.path).map_err(|why| WireError::Path {
            value: wire.path.clone(),
            why,
        })?,
        labels: wire.labels.iter().cloned().collect::<BTreeSet<_>>(),
        depends: wire
            .depends
            .iter()
            .map(|d| Oid::parse(d).map_err(oid("depends", d)))
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
            // The time records a completion, so an open task takes none.
            completed_at: t
                .completed_at
                .as_deref()
                .filter(|_| t.done)
                .map(|at| {
                    at.parse::<Timestamp>()
                        .map_err(|_| date("completed_at")(at.to_string()))
                })
                .transpose()?,
            priority: Priority::new(t.priority).map_err(|why| WireError::Priority {
                value: t.priority.to_string(),
                why,
            })?,
            due: t
                .due
                .as_deref()
                .map(parse_when)
                .transpose()
                .map_err(date("due"))?,
            deadline: t
                .deadline
                .as_deref()
                .map(|d| {
                    d.parse::<Date>()
                        .map_err(|_| date("deadline")(d.to_string()))
                })
                .transpose()?,
            event: t
                .event
                .as_deref()
                .map(|o| Oid::parse(o).map_err(oid("event", o)))
                .transpose()?,
        })),
        ("event", _, Some(e)) => Ok(Object::Event(Event {
            base,
            start: parse_when(&e.start).map_err(date("start"))?,
            end: parse_when(&e.end).map_err(date("end"))?,
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
                .collect::<Result<_, WireError>>()?,
            status: match e.status.as_str() {
                "confirmed" => EventStatus::Confirmed,
                "tentative" => EventStatus::Tentative,
                "cancelled" => EventStatus::Cancelled,
                v => return Err(unknown("status")(v)),
            },
            transparency: match e.transparency.as_str() {
                "busy" => Transparency::Busy,
                "free" => Transparency::Free,
                v => return Err(unknown("transparency")(v)),
            },
            visibility: match e.visibility.as_str() {
                "default" => Visibility::Default,
                "public" => Visibility::Public,
                "private" => Visibility::Private,
                "confidential" => Visibility::Confidential,
                v => return Err(unknown("visibility")(v)),
            },
            event_type: match e.event_type.as_str() {
                "default" => EventType::Default,
                "focus_time" => EventType::FocusTime,
                "out_of_office" => EventType::OutOfOffice,
                "working_location" => EventType::WorkingLocation,
                "birthday" => EventType::Birthday,
                v => return Err(unknown("event_type")(v)),
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
        (kind, _, _) => Err(WireError::UnknownKind(kind.to_string())),
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

fn parse_response(text: &str) -> Result<ResponseStatus, WireError> {
    Ok(match text {
        "needs_action" => ResponseStatus::NeedsAction,
        "accepted" => ResponseStatus::Accepted,
        "declined" => ResponseStatus::Declined,
        "tentative" => ResponseStatus::Tentative,
        v => {
            return Err(WireError::UnknownEnum {
                field: "response",
                value: v.to_string(),
            });
        }
    })
}
