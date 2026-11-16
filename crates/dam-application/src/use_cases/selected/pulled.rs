use std::cell::Cell;

use dam_domain::{Date, Object};
use jiff::tz::TimeZone;

use super::event_days::event_days;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeyondPulled {
    pub filters: Vec<String>,
    pub fields: Vec<&'static str>,
    pub held: Option<(Date, Date)>,
    pub tasks: usize,
    pub events: usize,
}

pub(super) struct Pulled {
    pub(super) held: Option<(Date, Date)>,
    placing: Cell<bool>,
}

impl Pulled {
    pub(super) fn of(seen: &[Object], zone: &TimeZone) -> Pulled {
        let days: Vec<Date> = seen
            .iter()
            .filter_map(Object::as_event)
            .flat_map(|e| event_days(e, zone))
            .collect();
        let held = days.iter().min().zip(days.iter().max());
        Pulled {
            held: held.map(|(first, last)| (*first, *last)),
            placing: Cell::new(false),
        }
    }

    pub(super) fn places_anyway(&self, day: Date) -> bool {
        let outside = !self
            .held
            .is_some_and(|(first, last)| first <= day && day <= last);
        self.placing.get() && outside
    }

    pub(super) fn placing(&self, on: bool) {
        self.placing.set(on);
    }
}
