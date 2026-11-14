mod event;
mod folder;
mod task;

use std::collections::BTreeSet;

use crate::vocabulary::vocabulary;
use crate::{Oid, Path, Recurrence};

pub use event::{
    Attachment, Attendee, Conference, Event, EventStatus, EventType, Person, ResponseStatus,
    Transparency, Visibility,
};
pub use task::{Completion, FOCUS_PER_DAY, Focus, Task};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Task,
    Event,
}

vocabulary!(Kind {
    Task => "task",
    Event => "event",
});

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reminder {
    pub minutes_before: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base {
    pub oid: Oid,
    pub subject: String,
    pub body: String,
    pub path: Path,
    pub labels: BTreeSet<String>,
    pub depends: Vec<Oid>,
    pub reminders: Vec<Reminder>,
    pub recurrence: Option<Recurrence>,
}

impl Base {
    pub(crate) fn new(oid: Oid, subject: impl Into<String>) -> Base {
        Base {
            oid,
            subject: subject.into(),
            body: String::new(),
            path: Path::root(),
            labels: BTreeSet::new(),
            depends: Vec::new(),
            reminders: Vec::new(),
            recurrence: None,
        }
    }
}

#[allow(
    clippy::large_enum_variant,
    reason = "both variants carry their full state directly, with no indirection"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Task(Task),
    Event(Event),
}

impl Object {
    pub fn oid(&self) -> &Oid {
        &self.base().oid
    }

    pub fn base(&self) -> &Base {
        match self {
            Object::Task(t) => &t.base,
            Object::Event(e) => &e.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut Base {
        match self {
            Object::Task(t) => &mut t.base,
            Object::Event(e) => &mut e.base,
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            Object::Task(_) => Kind::Task,
            Object::Event(_) => Kind::Event,
        }
    }

    pub fn as_task(&self) -> Option<&Task> {
        match self {
            Object::Task(t) => Some(t),
            Object::Event(_) => None,
        }
    }

    pub fn as_event(&self) -> Option<&Event> {
        match self {
            Object::Event(e) => Some(e),
            Object::Task(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
