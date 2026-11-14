use super::*;
use dam_application::Clock;

#[test]
fn today_is_the_date_of_now_in_the_system_zone() {
    let clock = SystemClock;
    let now = clock.now().to_zoned(jiff::tz::TimeZone::system());
    assert_eq!(clock.today(), now.date());
}
