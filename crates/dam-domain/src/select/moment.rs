use jiff::civil::{Date, ISOWeekDate, Weekday};
use jiff::{Span, ToSpan, Zoned};

use super::stretch::Stretch;
use crate::weekday::week_start;
use crate::{When, parse_minutes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unit {
    Day,
    Week,
    Month,
    Year,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Moment {
    Anchor(Unit, i32),
    Period(Unit, Date),
    Now(i64),
    Written(String),
}

impl Moment {
    pub(crate) fn read(text: &str) -> Option<Moment> {
        now(text)
            .or_else(|| anchor(text))
            .or_else(|| period(text))
            .or_else(|| written(text))
    }

    pub(crate) fn resolve(&self, now: &Zoned) -> Option<Stretch> {
        let today = now.date();
        match self {
            Moment::Anchor(unit, n) => {
                let start = unit_start(*unit, today).checked_add(step(*unit, *n)).ok()?;
                Stretch::whole(start, *unit, now.time_zone())
            }
            Moment::Period(unit, start) => Stretch::whole(*start, *unit, now.time_zone()),
            Moment::Now(minutes) => Stretch::instant(now.timestamp() + minutes.minutes()),
            Moment::Written(text) => match When::parse_human(text, today, now.time_zone()).ok()? {
                When::Day(day) => Stretch::whole(day, Unit::Day, now.time_zone()),
                When::At(at) => Stretch::instant(at.timestamp()),
            },
        }
    }

    pub(crate) fn is_one_day_or_instant(&self) -> bool {
        matches!(
            self,
            Moment::Anchor(Unit::Day, _) | Moment::Now(_) | Moment::Written(_)
        )
    }
}

pub(crate) fn step(unit: Unit, n: i32) -> Span {
    match unit {
        Unit::Day => n.days(),
        Unit::Week => n.weeks(),
        Unit::Month => n.months(),
        Unit::Year => n.years(),
    }
}

fn unit_start(unit: Unit, today: Date) -> Date {
    match unit {
        Unit::Day => today,
        Unit::Week => week_start(today),
        Unit::Month => today.first_of_month(),
        Unit::Year => today.first_of_year(),
    }
}

fn now(text: &str) -> Option<Moment> {
    let rest = text.strip_prefix("now")?;
    if rest.is_empty() {
        return Some(Moment::Now(0));
    }
    let (sign, length) = rest.split_at_checked(1)?;
    let minutes = i64::from(parse_minutes(length)?);
    match sign {
        "+" => Some(Moment::Now(minutes)),
        "-" => Some(Moment::Now(-minutes)),
        _ => None,
    }
}

fn anchor(text: &str) -> Option<Moment> {
    let at = text.find(['+', '-']).unwrap_or(text.len());
    let unit = match &text[..at] {
        "today" => Unit::Day,
        "week" => Unit::Week,
        "month" => Unit::Month,
        "year" => Unit::Year,
        _ => return None,
    };
    let offset = &text[at..];
    if offset.is_empty() {
        return Some(Moment::Anchor(unit, 0));
    }
    if !offset[1..].bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    offset.parse().ok().map(|n| Moment::Anchor(unit, n))
}

fn period(text: &str) -> Option<Moment> {
    let number = |s: &str, len: usize| {
        (s.len() == len && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse::<i16>().ok())
            .flatten()
    };
    let (year, rest) = text.split_at_checked(4)?;
    let year = number(year, 4)?;
    if rest.is_empty() {
        return Date::new(year, 1, 1)
            .ok()
            .map(|d| Moment::Period(Unit::Year, d));
    }
    if let Some(week) = rest.strip_prefix("-W") {
        let monday = ISOWeekDate::new(year, number(week, 2)? as i8, Weekday::Monday).ok()?;
        return Some(Moment::Period(Unit::Week, monday.date()));
    }
    let month = number(rest.strip_prefix('-')?, 2)?;
    Date::new(year, month as i8, 1)
        .ok()
        .map(|d| Moment::Period(Unit::Month, d))
}

fn written(text: &str) -> Option<Moment> {
    let any_day = Date::constant(2000, 1, 1);
    When::parse_human(text, any_day, &jiff::tz::TimeZone::UTC)
        .ok()
        .map(|_| Moment::Written(text.to_string()))
}
