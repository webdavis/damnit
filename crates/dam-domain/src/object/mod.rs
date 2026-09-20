mod event;
mod task;

use std::collections::BTreeSet;

use crate::{Oid, Path};
#[cfg(test)]
use crate::{Priority, When};

pub use event::{
    Attachment, Attendee, Conference, Event, EventStatus, EventType, Person, ResponseStatus,
    Transparency, Visibility,
};
pub use task::Task;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Task,
    Event,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Task => "task",
            Kind::Event => "event",
        }
    }

    pub fn parse(name: &str) -> Option<Kind> {
        match name {
            "task" => Some(Kind::Task),
            "event" => Some(Kind::Event),
            _ => None,
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reminder relative to the object's time: minutes before due or start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reminder {
    pub minutes_before: i64,
}

/// What every object has, whatever its kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base {
    pub oid: Oid,
    pub subject: String,
    pub body: String,
    pub path: Path,
    pub labels: BTreeSet<String>,
    pub depends: Vec<Oid>,
    pub reminders: Vec<Reminder>,
    /// The compact rule text; `recurrence::Rule::parse` reads it.
    pub recurrence: Option<String>,
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

/// Both variants carry their full state directly; no indirection.
#[allow(clippy::large_enum_variant)]
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
mod tests {
    use super::*;
    use crate::Oid;
    use jiff::civil::date;

    fn oid(byte: u8) -> Oid {
        Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
    }

    #[test]
    fn a_new_task_sits_at_root_open_and_lowest_priority() {
        let t = Task::new(oid(1), "buy milk");
        assert_eq!(t.base.subject, "buy milk");
        assert_eq!(t.base.path, Path::root());
        assert!(!t.done);
        assert_eq!(t.priority, Priority::LOWEST);
        assert!(t.due.is_none() && t.deadline.is_none() && t.event.is_none());
        assert!(t.base.labels.is_empty() && t.base.depends.is_empty());
    }

    #[test]
    fn a_new_event_is_confirmed_busy_and_default_typed() {
        let e = Event::new(
            oid(2),
            "dentist",
            When::Day(date(2026, 9, 25)),
            When::Day(date(2026, 9, 26)),
        );
        assert_eq!(e.status, EventStatus::Confirmed);
        assert_eq!(e.transparency, Transparency::Busy);
        assert_eq!(e.visibility, Visibility::Default);
        assert_eq!(e.event_type, EventType::Default);
        assert!(e.attendees.is_empty());
    }

    #[test]
    fn object_reports_its_kind_and_shared_base() {
        let t = Object::Task(Task::new(oid(3), "x"));
        let e = Object::Event(Event::new(
            oid(4),
            "y",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        ));
        assert_eq!(t.kind(), Kind::Task);
        assert_eq!(e.kind(), Kind::Event);
        assert_eq!(t.oid(), &oid(3));
        assert_eq!(e.base().subject, "y");
        assert!(t.as_task().is_some() && t.as_event().is_none());
        assert!(e.as_event().is_some() && e.as_task().is_none());
    }

    #[test]
    fn base_mut_edits_shared_fields_on_either_kind() {
        let mut t = Object::Task(Task::new(oid(5), "x"));
        t.base_mut().labels.insert("deep".to_string());
        assert!(t.base().labels.contains("deep"));
    }
}
