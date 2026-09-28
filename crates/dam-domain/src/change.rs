use std::fmt;

use crate::{Field, Object, Oid, Timestamp};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub oid: Oid,
    pub op: Op,
    pub before: Option<Object>,
    pub after: Option<Object>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommitId(Oid);

impl CommitId {
    pub fn generate(fill: &mut dyn FnMut(&mut [u8])) -> CommitId {
        CommitId(Oid::generate(fill))
    }

    pub fn parse(text: &str) -> Result<CommitId, crate::OidError> {
        Oid::parse(text).map(CommitId)
    }

    pub fn short(&self) -> &str {
        self.0.short()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRecord {
    pub id: CommitId,
    pub message: String,
    pub at: Timestamp,
    pub changes: Vec<Change>,
}

pub fn diff(oid: &Oid, before: Option<&Object>, after: Option<&Object>) -> Option<Change> {
    let op = match (before, after) {
        (None, None) => return None,
        (None, Some(_)) => Op::Create,
        (Some(_), None) => Op::Delete,
        (Some(b), Some(a)) if b == a => return None,
        (Some(_), Some(_)) => Op::Update,
    };
    Some(Change {
        oid: oid.clone(),
        op,
        before: before.cloned(),
        after: after.cloned(),
    })
}

pub fn coalesce(first: Change, second: Change) -> Option<Change> {
    let oid = first.oid.clone();
    match (first.op, second.op) {
        (Op::Create, Op::Delete) => None,
        (Op::Create, Op::Update) => Some(Change {
            oid,
            op: Op::Create,
            before: None,
            after: second.after,
        }),
        (Op::Update, Op::Update) => Some(Change {
            oid,
            op: Op::Update,
            before: first.before,
            after: second.after,
        }),
        (Op::Update, Op::Delete) => Some(Change {
            oid,
            op: Op::Delete,
            before: first.before,
            after: None,
        }),
        (Op::Delete, Op::Create) => diff(&oid, first.before.as_ref(), second.after.as_ref()),
        (_, _) => Some(second),
    }
}

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
            if b.done != a.done {
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
    match (&change.before, &change.after) {
        (Some(before), Some(after)) => changed_fields(before, after),
        (None, Some(after)) => fields_a_create_sets(after),
        _ => Vec::new(),
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod touched;
