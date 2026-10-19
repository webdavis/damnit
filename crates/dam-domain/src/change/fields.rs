use super::Change;
use crate::{Field, Object};

pub fn changed_fields(before: &Object, after: &Object) -> Vec<Field> {
    let mut out = Vec::new();
    let (b, a) = (before.base(), after.base());
    if b.subject != a.subject {
        out.push(Field::Subject);
    }
    if b.body != a.body {
        out.push(Field::Body);
    }
    if b.path != a.path {
        out.push(Field::Path);
    }
    if b.labels != a.labels {
        out.push(Field::Labels);
    }
    if b.depends != a.depends {
        out.push(Field::Depends);
    }
    if b.reminders != a.reminders {
        out.push(Field::Reminders);
    }
    if b.recurrence != a.recurrence {
        out.push(Field::Recurrence);
    }
    match (before, after) {
        (Object::Task(b), Object::Task(a)) => {
            if b.priority != a.priority {
                out.push(Field::Priority);
            }
            if b.completion != a.completion {
                out.push(Field::Done);
            }
            if b.due != a.due {
                out.push(Field::Due);
            }
            if b.deadline != a.deadline {
                out.push(Field::Deadline);
            }
            if b.event != a.event {
                out.push(Field::Event);
            }
        }
        (Object::Event(b), Object::Event(a)) => {
            if b.start != a.start {
                out.push(Field::Start);
            }
            if b.end != a.end {
                out.push(Field::End);
            }
            if b.timezone != a.timezone {
                out.push(Field::Timezone);
            }
            if b.location != a.location {
                out.push(Field::Location);
            }
            if b.attendees != a.attendees {
                out.push(Field::Attendees);
            }
            if b.status != a.status {
                out.push(Field::Status);
            }
            if b.transparency != a.transparency {
                out.push(Field::Transparency);
            }
            if b.visibility != a.visibility {
                out.push(Field::Visibility);
            }
            if b.event_type != a.event_type {
                out.push(Field::EventType);
            }
            if b.color != a.color {
                out.push(Field::Color);
            }
            if b.organizer != a.organizer {
                out.push(Field::Organizer);
            }
            if b.conference != a.conference {
                out.push(Field::Conference);
            }
            if b.attachments != a.attachments {
                out.push(Field::Attachments);
            }
        }
        _ => out.push(Field::Kind),
    }
    out
}

pub fn touched_fields(change: &Change) -> Vec<Field> {
    match change {
        Change::Update { before, after } => changed_fields(before, after),
        Change::Create { after } => fields_a_create_sets(after),
        Change::Delete { .. } => Vec::new(),
    }
}

fn fields_a_create_sets(object: &Object) -> Vec<Field> {
    let always_set: Vec<Field> = match object {
        Object::Task(_) => vec![Field::Subject],
        Object::Event(_) => vec![Field::Subject, Field::Start, Field::End],
    };
    let defaults = with_every_field_at_default(object);
    let non_default = changed_fields(&defaults, object)
        .into_iter()
        .filter(|field| !always_set.contains(field));
    always_set.iter().copied().chain(non_default).collect()
}

fn with_every_field_at_default(object: &Object) -> Object {
    let oid = object.base().oid.clone();
    match object {
        Object::Task(_) => Object::Task(crate::Task::new(oid, "")),
        Object::Event(e) => {
            Object::Event(crate::Event::new(oid, "", e.start.clone(), e.end.clone()))
        }
    }
}
