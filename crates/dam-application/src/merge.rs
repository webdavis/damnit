use dam_domain::{Field, Kind, Object};

/// The two kinds when a pulled object is not the kind we hold, which is what
/// `merge_fields` refuses to merge.
pub fn kind_change(ours: &Object, theirs: &Object) -> Option<(Kind, Kind)> {
    let (o, t) = (ours.kind(), theirs.kind());
    (o != t).then_some((o, t))
}

/// Copies the named fields from `theirs` onto a clone of `ours`. Anything not
/// named, which includes every dam-only field, keeps ours.
pub fn merge_fields(ours: &Object, theirs: &Object, fields: &[Field]) -> Object {
    let mut out = ours.clone();
    let has = |field: Field| fields.contains(&field);
    {
        let (o, t) = (out.base_mut(), theirs.base());
        if has(Field::Subject) {
            o.subject = t.subject.clone()
        }
        if has(Field::Body) {
            o.body = t.body.clone()
        }
        if has(Field::Path) {
            o.path = t.path.clone()
        }
        if has(Field::Labels) {
            o.labels = t.labels.clone()
        }
        if has(Field::Depends) {
            o.depends = t.depends.clone()
        }
        if has(Field::Reminders) {
            o.reminders = t.reminders.clone()
        }
        if has(Field::Recurrence) {
            o.recurrence = t.recurrence.clone()
        }
    }
    match (&mut out, theirs) {
        (Object::Task(o), Object::Task(t)) => {
            if has(Field::Done) {
                o.done = t.done
            }
            if has(Field::Priority) {
                o.priority = t.priority
            }
            if has(Field::Due) {
                o.due = t.due.clone()
            }
            if has(Field::Deadline) {
                o.deadline = t.deadline
            }
            if has(Field::Event) {
                o.event = t.event.clone()
            }
        }
        (Object::Event(o), Object::Event(t)) => {
            if has(Field::Start) {
                o.start = t.start.clone()
            }
            if has(Field::End) {
                o.end = t.end.clone()
            }
            if has(Field::Timezone) {
                o.timezone = t.timezone.clone()
            }
            if has(Field::Location) {
                o.location = t.location.clone()
            }
            if has(Field::Attendees) {
                o.attendees = t.attendees.clone()
            }
            if has(Field::Status) {
                o.status = t.status
            }
            if has(Field::Transparency) {
                o.transparency = t.transparency
            }
            if has(Field::Visibility) {
                o.visibility = t.visibility
            }
            if has(Field::EventType) {
                o.event_type = t.event_type
            }
            if has(Field::Color) {
                o.color = t.color.clone()
            }
            if has(Field::Organizer) {
                o.organizer = t.organizer.clone()
            }
            if has(Field::Conference) {
                o.conference = t.conference.clone()
            }
            if has(Field::Attachments) {
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
            &[Field::Subject],
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
        assert_eq!(merge_fields(&ours, &theirs, &[Field::Subject]), ours);
        assert_eq!(kind_change(&ours, &theirs), Some((Kind::Task, Kind::Event)));
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
            let merged = merge_fields(&ours, &theirs, &[*f]);
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
            let merged = merge_fields(&ours, &theirs, &[*f]);
            assert_eq!(changed_fields(&ours, &merged), vec![*f], "field {f}");
        }
    }
}
