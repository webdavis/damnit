use dam_domain::{Date, Timestamp};

pub trait Clock {
    fn today(&self) -> Date;
    fn now(&self) -> Timestamp;
}

pub trait Randomness {
    fn fill(&self, buf: &mut [u8]);
}
