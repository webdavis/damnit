use super::Base;
use crate::vocabulary::vocabulary;
use crate::{Oid, When};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

vocabulary!(EventStatus {
    Confirmed => "confirmed",
    Tentative => "tentative",
    Cancelled => "cancelled",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Transparency {
    Busy,
    Free,
}

vocabulary!(Transparency {
    Busy => "busy",
    Free => "free",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Visibility {
    Default,
    Public,
    Private,
    Confidential,
}

vocabulary!(Visibility {
    Default => "default",
    Public => "public",
    Private => "private",
    Confidential => "confidential",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    Default,
    FocusTime,
    OutOfOffice,
    WorkingLocation,
    Birthday,
}

vocabulary!(EventType {
    Default => "default",
    FocusTime => "focus_time",
    OutOfOffice => "out_of_office",
    WorkingLocation => "working_location",
    Birthday => "birthday",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResponseStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
}

vocabulary!(ResponseStatus {
    NeedsAction => "needs_action",
    Accepted => "accepted",
    Declined => "declined",
    Tentative => "tentative",
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attendee {
    pub email: String,
    pub response: ResponseStatus,
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

impl Event {
    pub fn span(&self, zone: &jiff::tz::TimeZone) -> Option<(crate::Timestamp, crate::Timestamp)> {
        Some((self.start.instant(zone)?, self.end.instant(zone)?))
    }

    pub fn holds_time(&self) -> bool {
        self.status != EventStatus::Cancelled
            && self.transparency == Transparency::Busy
            && !self
                .attendees
                .iter()
                .any(|a| a.is_self && a.response == ResponseStatus::Declined)
    }
}

#[cfg(test)]
mod tests;
