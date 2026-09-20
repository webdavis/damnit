use super::Base;
use crate::{Date, Oid, Priority, When};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub base: Base,
    pub done: bool,
    pub priority: Priority,
    pub due: Option<When>,
    pub deadline: Option<Date>,
    /// The event this task is attached to, by oid. dam-only data.
    pub event: Option<Oid>,
}

impl Task {
    pub fn new(oid: Oid, subject: impl Into<String>) -> Task {
        Task {
            base: Base::new(oid, subject),
            done: false,
            priority: Priority::default(),
            due: None,
            deadline: None,
            event: None,
        }
    }
}
