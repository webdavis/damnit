use super::Context;
use super::field::FieldType::*;
use super::field::Has::{Both, Event, Task};
use super::field::Holds::{Many, One, OneOrNone};
use super::field::{FieldType, Has, Holds, SelectField};
use super::readers::{base, event, task};
use super::values::Values;
use crate::{EventStatus, EventType, Object, ResponseStatus, Transparency, Visibility};

const fn row(
    key: &'static str,
    ty: FieldType,
    has: Has,
    holds: Holds,
    read: fn(&Object, &dyn Context) -> Values,
) -> SelectField {
    SelectField {
        key,
        ty,
        has,
        holds,
        read,
    }
}

pub static KIND: SelectField = row("kind", Choice(kinds), Both, One, base::kind);
pub static DONE: SelectField = row("done", YesNo, Task, One, task::done);
pub static COMPLETED_AT: SelectField =
    row("completed_at", Date, Task, OneOrNone, task::completed_at);

static FIELDS: [&SelectField; 31] = [
    &KIND,
    &row("subject", Text, Both, One, base::subject),
    &row("body", Text, Both, OneOrNone, base::body),
    &row("path", Path, Both, One, base::path),
    &row("labels", TextSet, Both, Many, base::labels),
    &row("category", Category, Both, Many, base::labels),
    &row("parent", Reference, Both, OneOrNone, base::parent),
    &row("depends", Reference, Both, Many, base::depends),
    &row("reminders", Length, Both, Many, base::reminders),
    &row("filter", FilterName, Both, One, base::own_id),
    &row("author", TextSet, Both, One, base::author),
    &DONE,
    &row("overdue", YesNo, Task, One, task::overdue),
    &row("blocked", YesNo, Task, One, task::blocked),
    &row("priority", Number(1, 4), Task, One, task::priority),
    &row("due", Date, Task, OneOrNone, task::due),
    &row("deadline", Date, Task, OneOrNone, task::deadline),
    &row("focus.day", Date, Task, OneOrNone, task::focus_day),
    &COMPLETED_AT,
    &row("estimate", Length, Task, OneOrNone, task::estimate),
    &row("attached", Reference, Task, OneOrNone, task::attached),
    &row("start", Date, Event, One, event::start),
    &row("end", Date, Event, One, event::end),
    &row("location", Text, Event, OneOrNone, event::location),
    &row("status", Choice(statuses), Event, One, event::status),
    &row(
        "transparency",
        Choice(transparencies),
        Event,
        One,
        event::transparency,
    ),
    &row(
        "visibility",
        Choice(visibilities),
        Event,
        One,
        event::visibility,
    ),
    &row(
        "event_type",
        Choice(event_types),
        Event,
        One,
        event::event_type,
    ),
    &row(
        "response",
        Choice(responses),
        Event,
        OneOrNone,
        event::response,
    ),
    &row(
        "organizer.email",
        TextSet,
        Event,
        OneOrNone,
        event::organizer_email,
    ),
    &row(
        "attendees.email",
        TextSet,
        Event,
        Many,
        event::attendee_emails,
    ),
];

pub fn select_fields() -> &'static [&'static SelectField] {
    &FIELDS
}

pub fn select_field(key: &str) -> Option<&'static SelectField> {
    FIELDS.iter().copied().find(|f| f.key == key)
}

fn kinds() -> Vec<&'static str> {
    let mut kinds: Vec<&str> = crate::Kind::ALL.iter().map(|k| k.as_str()).collect();
    kinds.push("any");
    kinds
}

fn statuses() -> Vec<&'static str> {
    EventStatus::ALL.iter().map(|v| v.as_str()).collect()
}

fn transparencies() -> Vec<&'static str> {
    Transparency::ALL.iter().map(|v| v.as_str()).collect()
}

fn visibilities() -> Vec<&'static str> {
    Visibility::ALL.iter().map(|v| v.as_str()).collect()
}

fn event_types() -> Vec<&'static str> {
    EventType::ALL.iter().map(|v| v.as_str()).collect()
}

fn responses() -> Vec<&'static str> {
    ResponseStatus::ALL.iter().map(|v| v.as_str()).collect()
}
