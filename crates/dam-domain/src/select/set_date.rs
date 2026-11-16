use jiff::{ToSpan, Zoned};

use super::date::DateTest;
use super::fields::select_field;
use super::moment::{Moment, Unit};
use crate::{Date, When};

pub fn date_to_set(text: &str, today: Date, now: &Zoned) -> Option<When> {
    match DateTest::read(text)?.one_moment()? {
        Moment::Anchor(Unit::Day, days) => today.checked_add(days.days()).ok().map(When::Day),
        Moment::Now(minutes) => now.checked_add(minutes.minutes()).ok().map(When::At),
        Moment::Written(text) => When::parse_human(text, today, now.time_zone()).ok(),
        Moment::Anchor(..) | Moment::Period(..) => None,
    }
}

pub fn date_to_set_shapes() -> Vec<&'static str> {
    let shapes = select_field("due").map_or_else(Vec::new, |f| f.date_shapes());
    shapes
        .into_iter()
        .filter(|s| DateTest::read(s).is_some_and(|d| d.is_one_day_or_instant()))
        .collect()
}
