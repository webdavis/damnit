use dam_domain::{Command, Event, Filter, Object, Path, Timestamp, WorkedOut, worked_out};

use crate::config::FilterConfig;

use crate::errors::UseCaseError;
use crate::service::Dam;

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
    pub worked_out: WorkedOut,
}

impl Dam<'_> {
    pub fn agenda(
        &self,
        filter: &Filter,
        place: &Path,
        local: &[FilterConfig],
        window: Window,
        zone: &jiff::tz::TimeZone,
    ) -> Result<Vec<Scheduled>, UseCaseError> {
        let (selected, facts) = self.selected(filter, Command::Agenda, place, local)?;
        let mut found: Vec<Scheduled> = selected
            .into_iter()
            .map(|o| (worked_out(&o, &facts), o))
            .filter_map(|(worked_out, o)| match o {
                Object::Event(event) => {
                    let (start, end) = event.span(zone)?;
                    (start < window.to && end > window.from).then(|| Scheduled {
                        worked_out,
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
}

#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod tests;
