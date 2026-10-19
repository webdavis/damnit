use std::collections::BTreeSet;

use dam_protocol::{PullResponse, RejectedObject};
use jiff::{SignedDuration, Timestamp};

use crate::ApiError;
use crate::calendar_api::{Calendar, Window};
use crate::map::{Mapped, map_event, remote_id};
use crate::memory::Memory;

pub const LOOKBACK_DAYS: i32 = 7;
pub const HORIZON_DAYS: i32 = 90;

pub fn window(now: Timestamp) -> Window {
    let day = SignedDuration::from_hours(24);
    Window {
        from: now.saturating_sub(day * LOOKBACK_DAYS).unwrap_or(now),
        to: now.saturating_add(day * HORIZON_DAYS).unwrap_or(now),
    }
}

pub fn pull(
    api: &impl Calendar,
    calendars: &[String],
    since: Option<&str>,
    now: Timestamp,
) -> Result<PullResponse, ApiError> {
    let window = window(now);
    let previous = Memory::read(since);
    let mut next = Memory::default();
    let mut seen = BTreeSet::new();
    let mut response = PullResponse::default();
    for calendar in calendars {
        let listing = api.events(calendar, &window)?;
        for event in &listing.items {
            match map_event(&listing, event) {
                Ok(Mapped::Live { object, end }) => {
                    if let Some(id) = &object.remote_id {
                        seen.insert(id.clone());
                        next.reported.insert(id.clone(), end);
                    }
                    response.objects.push(*object);
                }
                Ok(Mapped::Cancelled(id)) => {
                    seen.insert(id.clone());
                    response.cancelled.push(id);
                }
                Err(unreadable) => {
                    let id = remote_id(&listing.calendar, &event.id);
                    if let Some(end) = previous.reported.get(&id) {
                        next.reported.insert(id.clone(), *end);
                    }
                    seen.insert(id.clone());
                    response.rejected.push(RejectedObject {
                        remote_id: id,
                        why: unreadable.what.into(),
                    });
                }
            }
        }
    }
    response
        .cancelled
        .extend(previous.vanished(&seen, window.from.as_second()));
    response.sync = Some(next.token());
    Ok(response)
}

#[cfg(test)]
mod tests;
