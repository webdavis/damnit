use dam_domain::{Date, Timestamp};

pub trait Clock {
    fn today(&self) -> Date;
    fn now(&self) -> Timestamp;
    fn zone(&self) -> jiff::tz::TimeZone;
}

pub trait Interruption {
    fn requested(&self) -> bool;
}

pub trait Randomness {
    fn fill(&self, buf: &mut [u8]);
}
