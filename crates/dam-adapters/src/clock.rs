use dam_application::Clock;
use dam_domain::{Date, Timestamp};

pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> Date {
        jiff::Zoned::now().date()
    }

    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dam_application::Clock;

    #[test]
    fn today_is_the_date_of_now_in_the_system_zone() {
        let clock = SystemClock;
        let now = clock.now().to_zoned(jiff::tz::TimeZone::system());
        assert_eq!(clock.today(), now.date());
    }
}
