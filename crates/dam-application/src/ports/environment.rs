use dam_domain::{Date, Timestamp};

pub trait Clock {
    fn today(&self) -> Date;
    fn now(&self) -> Timestamp;
}

pub trait Randomness {
    fn fill(&self, buf: &mut [u8]);
}

#[derive(Debug, PartialEq, Eq)]
pub struct EditorError(pub String);

pub trait EditorSession {
    fn edit(&self, text: &str) -> Result<String, EditorError>;
}
