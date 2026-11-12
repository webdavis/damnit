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

pub const FOCUS_PER_DAY: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Focus {
    pub day: Date,
    pub set_by: String,
    pub set_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub base: Base,
    pub completion: Completion,
    pub priority: Priority,
    pub due: Option<When>,
    pub deadline: Option<Date>,
    pub event: Option<Oid>,
    pub estimate: Option<u32>,
    pub focus: Option<Focus>,
}

impl Task {
    pub fn focus_on(&self, day: Date) -> Option<&Focus> {
        self.focus.as_ref().filter(|f| f.day == day)
    }

    pub fn new(oid: Oid, subject: impl Into<String>) -> Task {
        Task {
            base: Base::new(oid, subject),
            completion: Completion::Open,
            priority: Priority::default(),
            due: None,
            deadline: None,
            event: None,
            estimate: None,
            focus: None,
        }
    }
}
