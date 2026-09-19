use dam_domain::Object;

use crate::wire::wire_kind;

/// The two wire kinds when a pulled object is not the kind we hold, which is
/// what `merge_fields` refuses to merge.
pub fn kind_change(ours: &Object, theirs: &Object) -> Option<(&'static str, &'static str)> {
    let (o, t) = (wire_kind(ours), wire_kind(theirs));
    (o != t).then_some((o, t))
}

/// Copies the named fields from `theirs` onto a clone of `ours`. Anything not
/// named, which includes every dam-only field, keeps ours.
pub fn merge_fields(ours: &Object, theirs: &Object, fields: &[String]) -> Object {
    let mut out = ours.clone();
    let has = |name: &str| fields.iter().any(|f| f == name);
    {
        let (o, t) = (out.base_mut(), theirs.base());
        if has("subject") {
            o.subject = t.subject.clone()
        }
        if has("body") {
            o.body = t.body.clone()
        }
        if has("path") {
            o.path = t.path.clone()
        }
        if has("labels") {
            o.labels = t.labels.clone()
        }
        if has("depends") {
            o.depends = t.depends.clone()
        }
        if has("reminders") {
            o.reminders = t.reminders.clone()
        }
        if has("recurrence") {
            o.recurrence = t.recurrence.clone()
        }
    }
    match (&mut out, theirs) {
        (Object::Task(o), Object::Task(t)) => {
            if has("done") {
                o.done = t.done
            }
            if has("priority") {
                o.priority = t.priority
            }
            if has("due") {
                o.due = t.due.clone()
            }
            if has("deadline") {
                o.deadline = t.deadline
            }
            if has("event") {
                o.event = t.event.clone()
            }
        }
        (Object::Event(o), Object::Event(t)) => {
            if has("start") {
                o.start = t.start.clone()
            }
            if has("end") {
                o.end = t.end.clone()
            }
            if has("timezone") {
                o.timezone = t.timezone.clone()
            }
            if has("location") {
                o.location = t.location.clone()
            }
            if has("attendees") {
                o.attendees = t.attendees.clone()
            }
            if has("status") {
                o.status = t.status
            }
            if has("transparency") {
                o.transparency = t.transparency
            }
            if has("visibility") {
                o.visibility = t.visibility
            }
            if has("event_type") {
                o.event_type = t.event_type
            }
            if has("color") {
                o.color = t.color.clone()
            }
            if has("organizer") {
                o.organizer = t.organizer.clone()
            }
            if has("conference") {
                o.conference = t.conference.clone()
            }
            if has("attachments") {
                o.attachments = t.attachments.clone()
            }
        }
        _ => return ours.clone(),
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::testing::oid;
    use dam_domain::{
        Attachment, Attendee, Conference, Event, EventStatus, EventType, Object, Path, Person,
        Priority, Reminder, ResponseStatus, Task, Transparency, Visibility, When, changed_fields,
    };
    use jiff::civil::date;

    #[test]
    fn only_named_fields_come_from_theirs() {
        let mut ours = Task::new(oid(1), "ours");
        ours.priority = Priority::HIGHEST;
        ours.base.depends.push(oid(9));
        let mut theirs = Task::new(oid(1), "theirs");
        theirs.priority = Priority::LOWEST;
        let out = merge_fields(
            &Object::Task(ours),
            &Object::Task(theirs),
            &["subject".into()],
        );
        let t = out.as_task().unwrap();
        assert_eq!(t.base.subject, "theirs");
        assert_eq!(t.priority, Priority::HIGHEST);
        assert_eq!(t.base.depends, vec![oid(9)]);
    }

    #[test]
    fn a_kind_mismatch_keeps_ours_entirely_and_is_reported() {
        let ours = Object::Task(Task::new(oid(1), "ours"));
        let theirs = Object::Event(Event::new(
            oid(1),
            "e",
            When::Day(date(2026, 1, 1)),
            When::Day(date(2026, 1, 2)),
        ));
        assert_eq!(merge_fields(&ours, &theirs, &["subject".into()]), ours);
        assert_eq!(kind_change(&ours, &theirs), Some(("task", "event")));
        assert_eq!(kind_change(&ours, &ours), None);
    }

    /// A task with every mergeable field set to a value distinct from its
    /// `"a"` counterpart, so `changed_fields` reports the whole set.
    fn task_variant(tag: &str) -> Task {
        let mut t = Task::new(oid(1), format!("subject-{tag}"));
        let n: u8 = if tag == "a" { 1 } else { 2 };
        t.base.body = format!("body-{tag}");
        t.base.path = Path::parse(&format!("area-{tag}")).unwrap();
        t.base.labels = BTreeSet::from([format!("label-{tag}")]);
        t.base.depends = vec![oid(90 + n)];
        t.base.reminders = vec![Reminder {
            minutes_before: n as i64,
        }];
        t.base.recurrence = Some(format!("every {n} days"));
        t.priority = if tag == "a" {
            Priority::HIGHEST
        } else {
            Priority::LOWEST
        };
        t.done = tag != "a";
        t.due = Some(When::Day(date(2026, 1, n as i8)));
        t.deadline = Some(date(2026, 2, n as i8));
        t.event = Some(oid(92 + n));
        t
    }

    /// An event with every mergeable field set to a value distinct from its
    /// `"a"` counterpart, so `changed_fields` reports the whole set.
    fn event_variant(tag: &str) -> Event {
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
        e.base.recurrence = Some(format!("every {n} days"));
        e.timezone = Some(format!("Zone/{tag}"));
        e.location = Some(format!("Room {tag}"));
        e.attendees = vec![Attendee {
            email: format!("{tag}@example.com"),
            response: ResponseStatus::Accepted,
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

    #[test]
    fn every_task_field_moves_alone_and_nothing_else_does() {
        let ours = Object::Task(task_variant("a"));
        let theirs = Object::Task(task_variant("b"));
        let fields = changed_fields(&ours, &theirs);
        assert_eq!(
            fields.len(),
            12,
            "expected every task field to differ: {fields:?}"
        );
        for f in &fields {
            let merged = merge_fields(&ours, &theirs, &[(*f).to_string()]);
            assert_eq!(changed_fields(&ours, &merged), vec![*f], "field {f}");
        }
    }

    #[test]
    fn every_event_field_moves_alone_and_nothing_else_does() {
        let ours = Object::Event(event_variant("a"));
        let theirs = Object::Event(event_variant("b"));
        let fields = changed_fields(&ours, &theirs);
        assert_eq!(
            fields.len(),
            20,
            "expected every event field to differ: {fields:?}"
        );
        for f in &fields {
            let merged = merge_fields(&ours, &theirs, &[(*f).to_string()]);
            assert_eq!(changed_fields(&ours, &merged), vec![*f], "field {f}");
        }
    }
}
