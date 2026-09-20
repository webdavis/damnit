//! What dam asks the world outside the process for: today's date, fresh
//! randomness, and the operator's editor.

use dam_domain::{Date, Timestamp};

pub trait Clock {
    fn today(&self) -> Date;
    fn now(&self) -> Timestamp;
}

/// A source of entropy for a new identifier. Shared rather than owned, so a
/// use case that mints one borrows it alongside everything else it reads.
pub trait Randomness {
    fn fill(&self, buf: &mut [u8]);
}

#[derive(Debug, PartialEq, Eq)]
pub struct EditorError(pub String);

pub trait EditorSession {
    fn edit(&self, text: &str) -> Result<String, EditorError>;
}
