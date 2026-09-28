use super::Base;
use crate::{Date, Oid, Priority, Timestamp, When};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub base: Base,
    pub done: bool,
    pub completed_at: Option<Timestamp>,
    pub priority: Priority,
    pub due: Option<When>,
    pub deadline: Option<Date>,
    pub event: Option<Oid>,
}

impl Task {
    pub fn new(oid: Oid, subject: impl Into<String>) -> Task {
        Task {
            base: Base::new(oid, subject),
            done: false,
            completed_at: None,
            priority: Priority::default(),
            due: None,
            deadline: None,
            event: None,
        }
    }
}
