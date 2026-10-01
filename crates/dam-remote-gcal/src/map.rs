mod people;
mod time;
mod words;

use std::fmt;

use dam_protocol::{WireEvent, WireObject};

use crate::calendar_api::{GoogleEvent, Listing};

#[derive(Debug)]
pub(crate) enum Mapped {
    Live { object: Box<WireObject>, end: i64 },
    Cancelled(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct MapError {
    pub calendar: String,
    pub event_id: String,
    pub what: &'static str,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "calendar {}, event {}: {}",
            self.calendar, self.event_id, self.what
        )
    }
}

impl std::error::Error for MapError {}

pub fn remote_id(calendar: &str, event_id: &str) -> String {
    format!("{calendar}/{event_id}")
}

pub(crate) fn map_event(listing: &Listing, event: &GoogleEvent) -> Result<Mapped, MapError> {
    let id = remote_id(&listing.calendar, &event.id);
    if event.status.as_deref() == Some("cancelled") {
        return Ok(Mapped::Cancelled(id));
    }
    let fail = |what| MapError {
        calendar: listing.calendar.clone(),
        event_id: event.id.clone(),
        what,
    };
    let zone = listing.time_zone.as_deref();
    let start = event
        .start
        .as_ref()
        .ok_or_else(|| fail("start is missing"))?;
    let end = event.end.as_ref().ok_or_else(|| fail("end is missing"))?;
    let (start_text, _) = time::when(start, zone).map_err(fail)?;
    let (end_text, end_second) = time::when(end, zone).map_err(fail)?;
    let wire_event = WireEvent {
        start: start_text,
        end: end_text,
        timezone: start.time_zone.clone(),
        location: event.location.clone(),
        attendees: people::attendees(event),
        status: words::status(event.status.as_deref()).into(),
        transparency: words::transparency(event.transparency.as_deref()).into(),
        visibility: words::visibility(event.visibility.as_deref()).into(),
        event_type: words::event_type(event.event_type.as_deref()).into(),
        color: event.color_id.clone(),
        organizer: people::organizer(event),
        conference: people::conference(event),
        attachments: people::attachments(event),
    };
    let object = WireObject {
        oid: String::new(),
        remote_id: Some(id),
        kind: "event".into(),
        subject: event.summary.clone().unwrap_or_default(),
        body: event.description.clone().unwrap_or_default(),
        path: words::path_segment(listing.summary.as_deref(), &listing.calendar),
        labels: vec![],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: None,
        event: Some(wire_event),
    };
    Ok(Mapped::Live {
        object: Box::new(object),
        end: end_second,
    })
}

#[cfg(test)]
mod tests;
