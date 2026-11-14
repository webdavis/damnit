use std::collections::BTreeSet;

use dam_adapters::{WireAttendee, WireConference, WirePerson, event_to_wire};
use dam_application::Scheduled;
use dam_domain::{Oid, Timestamp, When};
use jiff::tz::TimeZone;
use serde::Serialize;

use crate::view::display;

#[derive(Serialize)]
pub(super) struct AgendaEvent<'a> {
    #[serde(skip)]
    scheduled: &'a Scheduled,
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    subject: &'a str,
    path: &'a str,
    labels: &'a BTreeSet<String>,
    status: &'static str,
    transparency: &'static str,
    all_day: bool,
    start: String,
    end: String,
    busy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    attendees: Vec<WireAttendee>,
    #[serde(skip_serializing_if = "Option::is_none")]
    organizer: Option<WirePerson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conference: Option<WireConference>,
}

impl<'a> AgendaEvent<'a> {
    pub(super) fn new(s: &'a Scheduled, zone: &TimeZone) -> AgendaEvent<'a> {
        let base = &s.event.base;
        let all_day = s.event.start.is_all_day();
        let text = |t: Timestamp| {
            let local = t.to_zoned(zone.clone());
            if all_day {
                When::Day(local.date()).to_rfc3339()
            } else {
                When::At(local).to_rfc3339()
            }
        };
        let wire = event_to_wire(&s.event);
        AgendaEvent {
            scheduled: s,
            oid: &base.oid,
            subject: &base.subject,
            path: base.path.as_str(),
            labels: &base.labels,
            status: s.event.status.as_str(),
            transparency: s.event.transparency.as_str(),
            all_day,
            start: text(s.start),
            end: text(s.end),
            busy: s.busy,
            location: wire.location,
            attendees: wire.attendees,
            organizer: wire.organizer,
            conference: wire.conference,
        }
    }

    pub(super) fn cells(&self, zone: &TimeZone) -> Vec<String> {
        let s = self.scheduled;
        let local = |t: Timestamp| {
            let z = t.to_zoned(zone.clone());
            if self.all_day {
                z.strftime("%Y-%m-%d").to_string()
            } else {
                z.strftime("%Y-%m-%d %H:%M").to_string()
            }
        };
        vec![
            self.oid.short().to_string(),
            if self.busy { "busy" } else { "free" }.to_string(),
            local(s.start),
            local(s.end),
            self.path.to_string(),
            self.subject.to_string(),
        ]
    }
}
