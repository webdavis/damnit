use super::Base;
use crate::{Oid, When};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Transparency {
    Busy,
    Free,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Visibility {
    Default,
    Public,
    Private,
    Confidential,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    Default,
    FocusTime,
    OutOfOffice,
    WorkingLocation,
    Birthday,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResponseStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attendee {
    pub email: String,
    pub response: ResponseStatus,
    /// Whether this attendee is the calendar the event was read from:
    /// its answer is the calendar owner's.
    pub is_self: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conference {
    pub provider: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub url: String,
    pub title: String,
    pub mime_type: Option<String>,
}

/// An event has no `done`; it passes. Every field maps to one on Google
/// Calendar's event resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub base: Base,
    pub start: When,
    pub end: When,
    pub timezone: Option<String>,
    pub location: Option<String>,
    pub attendees: Vec<Attendee>,
    pub status: EventStatus,
    pub transparency: Transparency,
    pub visibility: Visibility,
    pub event_type: EventType,
    pub color: Option<String>,
    pub organizer: Option<Person>,
    pub conference: Option<Conference>,
    pub attachments: Vec<Attachment>,
}

impl Event {
    pub fn new(oid: Oid, subject: impl Into<String>, start: When, end: When) -> Event {
        Event {
            base: Base::new(oid, subject),
            start,
            end,
            timezone: None,
            location: None,
            attendees: Vec::new(),
            status: EventStatus::Confirmed,
            transparency: Transparency::Busy,
            visibility: Visibility::Default,
            event_type: EventType::Default,
            color: None,
            organizer: None,
            conference: None,
            attachments: Vec::new(),
        }
    }
}
