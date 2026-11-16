use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};

use super::moment::{Unit, step};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Stretch {
    pub from: Timestamp,
    pub until: Timestamp,
    pub whole: bool,
}

impl Stretch {
    pub(crate) fn whole(start: Date, unit: Unit, zone: &TimeZone) -> Option<Stretch> {
        let end = start.checked_add(step(unit, 1)).ok()?;
        Some(Stretch {
            from: start.to_zoned(zone.clone()).ok()?.timestamp(),
            until: end.to_zoned(zone.clone()).ok()?.timestamp(),
            whole: true,
        })
    }

    pub(crate) fn instant(at: Timestamp) -> Option<Stretch> {
        Some(Stretch {
            from: at,
            until: at.checked_add(1.minute()).ok()?,
            whole: false,
        })
    }

    pub(crate) fn end(&self) -> Timestamp {
        if self.whole { self.until } else { self.from }
    }

    pub(crate) fn holds(&self, point: Timestamp) -> bool {
        self.from <= point && point < self.until
    }
}
