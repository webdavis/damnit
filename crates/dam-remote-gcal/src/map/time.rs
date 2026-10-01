use jiff::civil::Date;
use jiff::tz::TimeZone;

use crate::calendar_api::GoogleTime;

fn zone_or_utc(name: Option<&str>) -> TimeZone {
    name.and_then(|n| TimeZone::get(n).ok())
        .unwrap_or(TimeZone::UTC)
}

pub(super) fn when(
    time: &GoogleTime,
    calendar_zone: Option<&str>,
) -> Result<(String, i64), &'static str> {
    if let Some(stated) = &time.date_time {
        let instant: jiff::Timestamp = stated.parse().map_err(|_| "a time is not RFC 3339")?;
        let zoned = instant.to_zoned(zone_or_utc(time.time_zone.as_deref().or(calendar_zone)));
        return Ok((zoned.to_string(), instant.as_second()));
    }
    if let Some(stated) = &time.date {
        let day: Date = stated.parse().map_err(|_| "a date is not YYYY-MM-DD")?;
        let midnight = day
            .to_zoned(zone_or_utc(calendar_zone))
            .map_err(|_| "a date is outside the calendar")?;
        return Ok((day.to_string(), midnight.timestamp().as_second()));
    }
    Err("a time names neither date nor dateTime")
}
