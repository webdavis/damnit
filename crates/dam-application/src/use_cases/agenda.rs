use dam_domain::{Event, Object, Timestamp};

use crate::config::Config;
use crate::errors::UseCaseError;
use crate::ports::{Clock, ObjectRepository};
use crate::use_cases::list::list;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: Timestamp,
    pub to: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheduled {
    pub event: Event,
    pub start: Timestamp,
    pub end: Timestamp,
    pub busy: bool,
}

pub fn agenda(
    objects: &dyn ObjectRepository,
    clock: &dyn Clock,
    config: &Config,
    query: Option<&str>,
    window: Window,
    zone: &jiff::tz::TimeZone,
) -> Result<Vec<Scheduled>, UseCaseError> {
    let mut found: Vec<Scheduled> = list(objects, clock, config, query)?
        .into_iter()
        .filter_map(|o| match o {
            Object::Event(event) => {
                let (start, end) = event.span(zone)?;
                (start < window.to && end > window.from).then(|| Scheduled {
                    busy: event.holds_time(),
                    event,
                    start,
                    end,
                })
            }
            Object::Task(_) => None,
        })
        .collect();
    found.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| a.event.base.oid.cmp(&b.event.base.oid))
    });
    Ok(found)
}

#[cfg(test)]
mod tests;
