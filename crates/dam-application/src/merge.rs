use dam_domain::Object;

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
    use super::*;
    use crate::testing::oid;
    use dam_domain::{Object, Priority, Task};

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
    fn a_kind_mismatch_keeps_ours_entirely() {
        let ours = Object::Task(Task::new(oid(1), "ours"));
        let theirs = Object::Event(dam_domain::Event::new(
            oid(1),
            "e",
            dam_domain::When::Day(jiff::civil::date(2026, 1, 1)),
            dam_domain::When::Day(jiff::civil::date(2026, 1, 2)),
        ));
        assert_eq!(merge_fields(&ours, &theirs, &["subject".into()]), ours);
    }
}
