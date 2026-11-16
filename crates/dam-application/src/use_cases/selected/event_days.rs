use dam_domain::{Date, Event, When};
use jiff::ToSpan;
use jiff::tz::TimeZone;

pub(super) fn event_days(event: &Event, zone: &TimeZone) -> Vec<Date> {
    let first = day_of(&event.start, zone);
    let last = match &event.end {
        When::Day(day) => day.yesterday().ok(),
        end => end
            .instant(zone)
            .and_then(|at| at.checked_sub(jiff::SignedDuration::from_nanos(1)).ok())
            .map(|at| at.to_zoned(zone.clone()).date()),
    };
    let (Some(first), last) = (first, last) else {
        return Vec::new();
    };
    let last = last.unwrap_or(first).max(first);
    first.series(1.day()).take_while(|d| *d <= last).collect()
}

fn day_of(when: &When, zone: &TimeZone) -> Option<Date> {
    match when {
        When::Day(day) => Some(*day),
        at => at.instant(zone).map(|at| at.to_zoned(zone.clone()).date()),
    }
}
