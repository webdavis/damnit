use dam_domain::{Completion, Event, EventStatus, Object, Reminder, Task, Timestamp, When};
use jiff::civil::date;
use jiff::tz::TimeZone;

use super::*;
use crate::testing::{Scenario, oid};

fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn zoned(text: &str) -> When {
    When::At(at(text).to_zoned(TimeZone::UTC))
}

fn reminded(minutes: &[i64]) -> Vec<Reminder> {
    minutes
        .iter()
        .map(|m| Reminder { minutes_before: *m })
        .collect()
}

fn task(n: u8, subject: &str, due: When, minutes: &[i64]) -> Task {
    let mut t = Task::new(oid(n), subject);
    t.due = Some(due);
    t.base.reminders = reminded(minutes);
    t
}

fn due_between(s: &Scenario, from: &str, to: &str) -> Vec<(String, Timestamp)> {
    let window = Window {
        from: at(from),
        to: at(to),
    };
    s.dam()
        .due_reminders(window, &TimeZone::UTC)
        .unwrap()
        .into_iter()
        .map(|r| (r.subject, r.at))
        .collect()
}

#[test]
fn reminders_fall_due_before_their_anchor_earliest_first_inside_a_half_open_window() {
    let s = Scenario::default();
    s.working(Object::Task(task(
        1,
        "call",
        zoned("2026-01-02T10:00:00Z"),
        &[30, 120],
    )));
    let mut meeting = Event::new(
        oid(2),
        "standup",
        zoned("2026-01-02T09:45:00Z"),
        zoned("2026-01-02T10:00:00Z"),
    );
    meeting.base.reminders = reminded(&[5]);
    s.working(Object::Event(meeting));
    s.working(Object::Task(task(
        3,
        "all day",
        When::Day(date(2026, 1, 2)),
        &[0],
    )));
    assert_eq!(
        due_between(&s, "2026-01-02T00:00:00Z", "2026-01-02T09:40:00Z"),
        vec![
            ("all day".to_string(), at("2026-01-02T00:00:00Z")),
            ("call".to_string(), at("2026-01-02T08:00:00Z")),
            ("call".to_string(), at("2026-01-02T09:30:00Z")),
        ]
    );
    assert_eq!(
        due_between(&s, "2026-01-02T09:40:00Z", "2026-01-02T12:00:00Z"),
        vec![("standup".to_string(), at("2026-01-02T09:40:00Z"))]
    );
}

#[test]
fn a_done_task_and_a_cancelled_event_raise_no_reminder() {
    let s = Scenario::default();
    let mut done = task(1, "done", zoned("2026-01-02T10:00:00Z"), &[30]);
    done.completion = Completion::Done { at: None };
    s.working(Object::Task(done));
    let mut cancelled = Event::new(
        oid(2),
        "cancelled",
        zoned("2026-01-02T10:00:00Z"),
        zoned("2026-01-02T11:00:00Z"),
    );
    cancelled.status = EventStatus::Cancelled;
    cancelled.base.reminders = reminded(&[30]);
    s.working(Object::Event(cancelled));
    assert!(due_between(&s, "2026-01-01T00:00:00Z", "2026-01-03T00:00:00Z").is_empty());
}
