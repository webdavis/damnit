use std::collections::BTreeSet;
use std::fmt;

use dam_protocol::PullResponse;
use jiff::{SignedDuration, Timestamp};

use crate::ApiError;
use crate::calendar_api::{CalendarApi, Window};
use crate::map::{MapError, Mapped, map_event};
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

#[derive(Debug)]
pub enum PullError {
    Api(ApiError),
    Map(MapError),
}

impl fmt::Display for PullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PullError::Api(e) => e.fmt(f),
            PullError::Map(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for PullError {}

pub fn pull(
    api: &CalendarApi,
    calendars: &[String],
    since: Option<&str>,
    now: Timestamp,
) -> Result<PullResponse, PullError> {
    let window = window(now);
    let previous = Memory::read(since);
    let mut next = Memory::default();
    let mut seen = BTreeSet::new();
    let mut response = PullResponse::default();
    for calendar in calendars {
        let listing = api.events(calendar, &window).map_err(PullError::Api)?;
        for event in &listing.items {
            match map_event(&listing, event).map_err(PullError::Map)? {
                Mapped::Live { object, end } => {
                    if let Some(id) = &object.remote_id {
                        seen.insert(id.clone());
                        next.reported.insert(id.clone(), end);
                    }
                    response.objects.push(*object);
                }
                Mapped::Cancelled(id) => {
                    seen.insert(id.clone());
                    response.cancelled.push(id);
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
