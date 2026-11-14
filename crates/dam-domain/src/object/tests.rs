use super::*;
use crate::{Oid, Priority, When};
use jiff::civil::date;

fn oid(byte: u8) -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(byte))
}

#[test]
fn a_new_task_sits_at_root_open_and_lowest_priority() {
    let t = Task::new(oid(1), "buy milk");
    assert_eq!(t.base.subject, "buy milk");
    assert_eq!(t.base.path, Path::root());
    assert_eq!(t.completion, Completion::Open);
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
