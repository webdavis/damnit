use dam_domain::{EventStatus, Kind, Object, Oid, Timestamp};
use jiff::SignedDuration;
use jiff::tz::TimeZone;

use crate::errors::UseCaseError;
use crate::service::Dam;
use crate::use_cases::agenda::Window;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DueReminder {
    pub oid: Oid,
    pub kind: Kind,
    pub subject: String,
    pub at: Timestamp,
    pub anchor: Timestamp,
    pub minutes_before: i64,
}

impl Dam<'_> {
    pub fn due_reminders(
        &self,
        window: Window,
        zone: &TimeZone,
    ) -> Result<Vec<DueReminder>, UseCaseError> {
        let mut due: Vec<DueReminder> = self
            .list(None)?
            .iter()
            .filter_map(|o| Some((o, anchor(o, zone)?)))
            .flat_map(|(o, anchor)| {
                o.base().reminders.iter().filter_map(move |r| {
                    let before = SignedDuration::from_mins(r.minutes_before);
                    Some(DueReminder {
                        oid: o.oid().clone(),
                        kind: o.kind(),
                        subject: o.base().subject.clone(),
                        at: anchor.checked_sub(before).ok()?,
                        anchor,
                        minutes_before: r.minutes_before,
                    })
                })
            })
            .filter(|r| window.from <= r.at && r.at < window.to)
            .collect();
        due.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.oid.cmp(&b.oid)));
        Ok(due)
    }
}

fn anchor(object: &Object, zone: &TimeZone) -> Option<Timestamp> {
    match object {
        Object::Task(t) if !t.completion.is_done() => t.due.as_ref()?.instant(zone),
        Object::Event(e) if e.status != EventStatus::Cancelled => e.start.instant(zone),
        _ => None,
    }
}

#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod tests;
