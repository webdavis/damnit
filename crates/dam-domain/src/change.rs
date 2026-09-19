use std::fmt;

use crate::{Object, Oid, Timestamp};

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

/// Names a commit the way an Oid names an object.
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

/// None when nothing differs.
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

/// Two changes to one oid in sequence collapse into one; a create followed by
/// a delete collapses to nothing.
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

/// Field names that differ, for status and for the helper field filter.
/// Empty for a create or delete.
pub fn changed_fields(before: &Object, after: &Object) -> Vec<&'static str> {
    let mut out = Vec::new();
    let (b, a) = (before.base(), after.base());
    if b.subject != a.subject {
        out.push("subject");
    }
    if b.body != a.body {
        out.push("body");
    }
    if b.path != a.path {
        out.push("path");
    }
    if b.labels != a.labels {
        out.push("labels");
    }
    if b.depends != a.depends {
        out.push("depends");
    }
    if b.reminders != a.reminders {
        out.push("reminders");
    }
    if b.recurrence != a.recurrence {
        out.push("recurrence");
    }
    match (before, after) {
        (Object::Task(b), Object::Task(a)) => {
            if b.priority != a.priority {
                out.push("priority");
            }
            if b.done != a.done {
                out.push("done");
            }
            if b.due != a.due {
                out.push("due");
            }
            if b.deadline != a.deadline {
                out.push("deadline");
            }
            if b.event != a.event {
                out.push("event");
            }
        }
        (Object::Event(b), Object::Event(a)) => {
            if b.start != a.start {
                out.push("start");
            }
            if b.end != a.end {
                out.push("end");
            }
            if b.timezone != a.timezone {
                out.push("timezone");
            }
            if b.location != a.location {
                out.push("location");
            }
            if b.attendees != a.attendees {
                out.push("attendees");
            }
            if b.status != a.status {
                out.push("status");
            }
            if b.transparency != a.transparency {
                out.push("transparency");
            }
            if b.visibility != a.visibility {
                out.push("visibility");
            }
            if b.event_type != a.event_type {
                out.push("event_type");
            }
            if b.color != a.color {
                out.push("color");
            }
            if b.organizer != a.organizer {
                out.push("organizer");
            }
            if b.conference != a.conference {
                out.push("conference");
            }
            if b.attachments != a.attachments {
                out.push("attachments");
            }
        }
        _ => out.push("kind"),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Object, Oid, Priority, Task};

    fn oid(b: u8) -> Oid {
        Oid::generate(&mut |x: &mut [u8]| x.fill(b))
    }

    fn task(subject: &str) -> Object {
        Object::Task(Task::new(oid(1), subject))
    }

    #[test]
    fn diff_reports_create_update_delete_and_nothing() {
        let a = task("a");
        let b = task("b");
        assert_eq!(diff(&oid(1), None, Some(&a)).unwrap().op, Op::Create);
        assert_eq!(diff(&oid(1), Some(&a), Some(&b)).unwrap().op, Op::Update);
        assert_eq!(diff(&oid(1), Some(&a), None).unwrap().op, Op::Delete);
        assert!(diff(&oid(1), Some(&a), Some(&a)).is_none());
        assert!(diff(&oid(1), None, None).is_none());
    }

    #[test]
    fn coalesce_create_then_update_is_a_create_of_the_final_state() {
        let a = task("a");
        let b = task("b");
        let c = diff(&oid(1), None, Some(&a)).unwrap();
        let u = diff(&oid(1), Some(&a), Some(&b)).unwrap();
        let out = coalesce(c, u).unwrap();
        assert_eq!(out.op, Op::Create);
        assert_eq!(out.after, Some(b));
    }

    #[test]
    fn coalesce_create_then_delete_is_nothing() {
        let a = task("a");
        let c = diff(&oid(1), None, Some(&a)).unwrap();
        let d = diff(&oid(1), Some(&a), None).unwrap();
        assert!(coalesce(c, d).is_none());
    }

    #[test]
    fn coalesce_update_then_update_keeps_the_first_before() {
        let a = task("a");
        let b = task("b");
        let c = task("c");
        let u1 = diff(&oid(1), Some(&a), Some(&b)).unwrap();
        let u2 = diff(&oid(1), Some(&b), Some(&c)).unwrap();
        let out = coalesce(u1, u2).unwrap();
        assert_eq!(out.op, Op::Update);
        assert_eq!(out.before, Some(a));
        assert_eq!(out.after, Some(c));
    }

    #[test]
    fn coalesce_update_then_delete_is_a_delete_of_the_original() {
        let a = task("a");
        let b = task("b");
        let u = diff(&oid(1), Some(&a), Some(&b)).unwrap();
        let d = diff(&oid(1), Some(&b), None).unwrap();
        let out = coalesce(u, d).unwrap();
        assert_eq!(out.op, Op::Delete);
        assert_eq!(out.before, Some(a));
    }

    #[test]
    fn changed_fields_names_what_moved() {
        let a = task("a");
        let mut t = Task::new(oid(1), "a");
        t.priority = Priority::HIGHEST;
        t.done = true;
        let b = Object::Task(t);
        assert_eq!(changed_fields(&a, &b), vec!["priority", "done"]);
        assert!(changed_fields(&a, &a).is_empty());
    }
}
