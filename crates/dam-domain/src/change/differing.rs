use std::collections::BTreeSet;

use crate::{
    Attachment, Attendee, Completion, Conference, Event, EventStatus, EventType, Oid, Path, Person,
    Priority, Recurrence, Reminder, ResponseStatus, Task, Transparency, Visibility, When,
};
use jiff::civil::date;

pub(super) fn oid(b: u8) -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(b))
}

pub(super) fn task_differing_in_every_field(tag: &str) -> Task {
    let mut t = Task::new(oid(1), format!("subject-{tag}"));
    let n: u8 = if tag == "a" { 1 } else { 2 };
    t.base.body = format!("body-{tag}");
    t.base.path = Path::parse(&format!("area-{tag}")).unwrap();
    t.base.labels = BTreeSet::from([format!("label-{tag}")]);
    t.base.depends = vec![oid(90 + n)];
    t.base.reminders = vec![Reminder {
        minutes_before: n as i64,
    }];
    t.base.recurrence = Some(Recurrence::parse(&format!("every {n} days")));
    t.priority = if tag == "a" {
        Priority::HIGHEST
    } else {
        Priority::LOWEST
    };
    t.completion = if tag == "a" {
        Completion::Open
    } else {
        Completion::Done { at: None }
    };
    t.due = Some(When::Day(date(2026, 1, n as i8)));
    t.deadline = Some(date(2026, 2, n as i8));
    t.event = Some(oid(92 + n));
    t
}

pub(super) fn event_differing_in_every_field(tag: &str) -> Event {
    let n: u8 = if tag == "a" { 1 } else { 2 };
    let mut e = Event::new(
        oid(1),
        format!("subject-{tag}"),
        When::Day(date(2026, 1, n as i8)),
        When::Day(date(2026, 1, n as i8 + 1)),
    );
    e.base.body = format!("body-{tag}");
    e.base.path = Path::parse(&format!("area-{tag}")).unwrap();
    e.base.labels = BTreeSet::from([format!("label-{tag}")]);
    e.base.depends = vec![oid(90 + n)];
    e.base.reminders = vec![Reminder {
        minutes_before: n as i64,
    }];
    e.base.recurrence = Some(Recurrence::parse(&format!("every {n} days")));
    e.timezone = Some(format!("Zone/{tag}"));
    e.location = Some(format!("Room {tag}"));
    e.attendees = vec![Attendee {
        email: format!("{tag}@example.com"),
        response: ResponseStatus::Accepted,
        is_self: false,
    }];
    e.status = if tag == "a" {
        EventStatus::Confirmed
    } else {
        EventStatus::Tentative
    };
    e.transparency = if tag == "a" {
        Transparency::Busy
    } else {
        Transparency::Free
    };
    e.visibility = if tag == "a" {
        Visibility::Default
    } else {
        Visibility::Private
    };
    e.event_type = if tag == "a" {
        EventType::Default
    } else {
        EventType::FocusTime
    };
    e.color = Some(format!("color-{tag}"));
    e.organizer = Some(Person {
        email: format!("org-{tag}@example.com"),
        name: None,
    });
    e.conference = Some(Conference {
        provider: format!("provider-{tag}"),
        url: format!("url-{tag}"),
    });
    e.attachments = vec![Attachment {
        url: format!("att-{tag}"),
        title: format!("title-{tag}"),
        mime_type: None,
    }];
    e
}
