use jiff::Zoned;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;

use crate::api::Due;

const ZONE_OF_A_BARE_OFFSET: &str = "UTC";

pub fn todoist_due(due: &str) -> serde_json::Value {
    match due.parse::<Zoned>() {
        Ok(held) => serde_json::json!({
            "date": held.timestamp().strftime("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "timezone": zone_name(due).unwrap_or_else(|| ZONE_OF_A_BARE_OFFSET.into()),
        }),
        Err(_) => serde_json::json!({ "date": due }),
    }
}

pub fn zone_name(due: &str) -> Option<String> {
    let held = due.parse::<Zoned>().ok()?;
    held.time_zone().iana_name().map(str::to_string)
}

pub fn dam_due(due: &Due, local: &TimeZone) -> String {
    let date = due.date.as_str();
    if let Ok(instant) = date.parse::<jiff::Timestamp>() {
        let zone = due
            .timezone
            .as_deref()
            .and_then(|name| TimeZone::get(name).ok())
            .unwrap_or(TimeZone::UTC);
        return instant.to_zoned(zone).to_string();
    }
    match date.parse::<DateTime>() {
        Ok(clock) if date.contains('T') => clock
            .to_zoned(local.clone())
            .map_or_else(|_| date.to_string(), |z| z.to_string()),
        _ => date.to_string(),
    }
}
