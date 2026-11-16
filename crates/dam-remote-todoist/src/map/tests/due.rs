use jiff::tz::TimeZone;
use serde_json::json;

use crate::api::Due;
use crate::map::{dam_due, todoist_due, zone_name};

fn todoist(date: &str, timezone: Option<&str>) -> Due {
    Due {
        date: date.into(),
        is_recurring: false,
        string: String::new(),
        timezone: timezone.map(str::to_string),
    }
}

#[test]
fn a_full_day_goes_as_its_date() {
    assert_eq!(todoist_due("2026-12-04"), json!({"date": "2026-12-04"}));
    assert_eq!(zone_name("2026-12-04"), None);
}

#[test]
fn a_time_goes_as_the_instant_in_utc_with_the_zone_it_is_held_in() {
    for (dam, utc) in [
        (
            "2026-12-04T20:00:00-05:00[America/New_York]",
            "2026-12-05T01:00:00Z",
        ),
        (
            "2027-06-04T20:00:00-04:00[America/New_York]",
            "2027-06-05T00:00:00Z",
        ),
        (
            "2026-12-04T20:00:30.5-05:00[America/New_York]",
            "2026-12-05T01:00:30Z",
        ),
    ] {
        assert_eq!(
            todoist_due(dam),
            json!({"date": utc, "timezone": "America/New_York"}),
            "{dam}"
        );
    }
    assert_eq!(
        zone_name("2026-12-04T20:00:00-05:00[America/New_York]").as_deref(),
        Some("America/New_York")
    );
}

#[test]
fn a_time_held_at_a_bare_offset_goes_as_the_same_instant_in_utc() {
    let dam = "2026-12-04T20:00:00-05:00[-05:00]";
    assert_eq!(
        todoist_due(dam),
        json!({"date": "2026-12-05T01:00:00Z", "timezone": "UTC"})
    );
    assert_eq!(zone_name(dam), None);
}

#[test]
fn a_fixed_zone_due_comes_in_as_that_instant_in_its_zone() {
    let local = TimeZone::get("Asia/Tokyo").unwrap();
    assert_eq!(
        dam_due(
            &todoist("2026-12-05T01:00:00.000000Z", Some("America/New_York")),
            &local
        ),
        "2026-12-04T20:00:00-05:00[America/New_York]"
    );
    assert_eq!(
        dam_due(&todoist("2026-12-05T01:00:00Z", None), &local),
        "2026-12-05T01:00:00+00:00[UTC]"
    );
}

#[test]
fn a_floating_time_comes_in_as_that_clock_time_in_the_persons_zone() {
    let local = TimeZone::get("Europe/Madrid").unwrap();
    assert_eq!(
        dam_due(&todoist("2026-12-04T10:00:00.000000", None), &local),
        "2026-12-04T10:00:00+01:00[Europe/Madrid]"
    );
}

#[test]
fn a_full_day_or_unreadable_date_comes_in_as_todoist_wrote_it() {
    let local = TimeZone::UTC;
    assert_eq!(dam_due(&todoist("2026-12-04", None), &local), "2026-12-04");
    assert_eq!(dam_due(&todoist("someday", None), &local), "someday");
}
