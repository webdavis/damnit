use super::Base;
use crate::{Date, Oid, Priority, Timestamp, When};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Completion {
    Open,
    Done { at: Option<Timestamp> },
}

impl Completion {
    pub fn is_done(&self) -> bool {
        matches!(self, Completion::Done { .. })
    }

    pub fn at(&self) -> Option<Timestamp> {
        match self {
            Completion::Done { at } => *at,
            Completion::Open => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub base: Base,
    pub completion: Completion,
    pub priority: Priority,
    pub due: Option<When>,
    pub deadline: Option<Date>,
    pub event: Option<Oid>,
}

impl Task {
    pub fn new(oid: Oid, subject: impl Into<String>) -> Task {
        Task {
            base: Base::new(oid, subject),
            completion: Completion::Open,
            priority: Priority::default(),
            due: None,
            deadline: None,
            event: None,
        }
    }
}
