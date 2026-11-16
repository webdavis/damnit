use dam_domain::{
    Command, Event, Filter, Object, Path, Scope, Task, Timestamp, WorkedOut, worked_out,
};
use jiff::civil::Date;
use jiff::tz::TimeZone;

use crate::config::FilterConfig;

use super::selected::Defaults;
use crate::errors::UseCaseError;
use crate::service::Dam;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: Timestamp,
    pub to: Timestamp,
}

impl Window {
    pub fn days(&self, zone: &TimeZone) -> (Date, Date) {
        let day = |t: Timestamp| t.to_zoned(zone.clone()).date();
        let last = self.to.checked_sub(jiff::SignedDuration::from_nanos(1));
        (day(self.from), day(last.unwrap_or(self.to)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheduled {
    pub event: Event,
    pub start: Timestamp,
    pub end: Timestamp,
    pub day: Date,
    pub busy: bool,
    pub worked_out: WorkedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Due {
    pub task: Task,
    pub day: Date,
    pub worked_out: WorkedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agenda {
    pub scope: Scope,
    pub events: Vec<Scheduled>,
    pub tasks: Vec<Due>,
}

impl Dam<'_> {
    pub fn agenda(
        &self,
        filter: &Filter,
        place: &Path,
        local: &[FilterConfig],
        window: Window,
        zone: &TimeZone,
    ) -> Result<Agenda, UseCaseError> {
        let scope = self.scope_once(filter, Command::Agenda, place, local)?;
        let (selected, facts) = self.chosen(filter, Defaults::Fixed(scope), place, local, &[])?;
        let (first, last) = window.days(zone);
        let mut found = Agenda {
            scope,
            events: Vec::new(),
            tasks: Vec::new(),
        };
        for object in selected {
            let worked_out = worked_out(&object, &facts);
            match object {
                Object::Event(event) => {
                    let Some((start, end)) = event.span(zone) else {
                        continue;
                    };
                    if start < window.to && end > window.from {
                        found.events.push(Scheduled {
                            worked_out,
                            busy: event.holds_time(),
                            day: start.to_zoned(zone.clone()).date(),
                            event,
                            start,
                            end,
                        });
                    }
                }
                Object::Task(task) => {
                    let day = task.due.as_ref().map(|d| d.day_in(zone));
                    if let Some(day) = day.filter(|d| (first..=last).contains(d)) {
                        found.tasks.push(Due {
                            task,
                            day,
                            worked_out,
                        });
                    }
                }
            }
        }
        found.events.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| a.event.base.oid.cmp(&b.event.base.oid))
        });
        found.tasks.sort_by_key(|d| d.day);
        Ok(found)
    }
}

#[cfg(test)]
mod access_tests;
#[cfg(test)]
mod tasks_tests;
#[cfg(test)]
mod tests;
