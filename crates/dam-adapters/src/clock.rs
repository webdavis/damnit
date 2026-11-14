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

    fn zone(&self) -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::system()
    }
}

#[cfg(test)]
mod tests;
