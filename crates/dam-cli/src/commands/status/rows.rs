use std::collections::BTreeSet;

use dam_application::Conflict;
use dam_domain::{Change, Field, Object, Oid, Timestamp, When, changed_fields, touched_fields};
use serde::Serialize;

use crate::view::{display, optional_display, optional_wire, wire};

pub(crate) fn change_line(change: &Change) -> String {
    let subject = &change.object().base().subject;
    let oid = change.oid().short();
    match change {
        Change::Create { .. } => format!("new      {oid}  {subject}"),
        Change::Delete { .. } => format!("removed  {oid}  {subject}"),
        Change::Update { before, after } => {
            let fields = changed_fields(before, after)
                .iter()
                .map(|f| f.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("changed  {oid}  {subject}  ({fields})")
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ChangeRow<'a> {
    #[serde(skip)]
    change: &'a Change,
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    op: &'static str,
    fields: Vec<&'static str>,
    kind: &'static str,
    subject: &'a str,
    path: &'a str,
    labels: &'a BTreeSet<String>,
    #[serde(flatten)]
    state: KindState,
    #[serde(flatten)]
    sides: Option<Sides<'a>>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum KindState {
    Task {
        done: bool,
        #[serde(serialize_with = "optional_display")]
        completed_at: Option<Timestamp>,
        priority: u8,
        due: Option<String>,
    },
    Event {
        start: String,
        end: String,
    },
}

#[derive(Serialize)]
struct Sides<'a> {
    #[serde(serialize_with = "optional_wire")]
    before: Option<&'a Object>,
    #[serde(serialize_with = "optional_wire")]
    after: Option<&'a Object>,
}

impl<'a> ChangeRow<'a> {
    pub(crate) fn new(change: &'a Change, full: bool) -> ChangeRow<'a> {
        let object = change.object();
        let base = object.base();
        ChangeRow {
            change,
            oid: change.oid(),
            op: change.op().as_str(),
            fields: touched_fields(change).iter().map(Field::as_str).collect(),
            kind: object.kind().as_str(),
            subject: &base.subject,
            path: base.path.as_str(),
            labels: &base.labels,
            state: match object {
                Object::Task(t) => KindState::Task {
                    done: t.completion.is_done(),
                    completed_at: t.completion.at(),
                    priority: t.priority.get(),
                    due: t.due.as_ref().map(When::to_text),
                },
                Object::Event(e) => KindState::Event {
                    start: e.start.to_text(),
                    end: e.end.to_text(),
                },
            },
            sides: full.then(|| Sides {
                before: change.before(),
                after: change.after(),
            }),
        }
    }

    pub(crate) fn line(&self) -> String {
        change_line(self.change)
    }
}

#[derive(Serialize)]
pub(super) struct ConflictRow<'a> {
    #[serde(serialize_with = "display")]
    oid: &'a Oid,
    remote: &'a str,
    #[serde(serialize_with = "wire")]
    ours: &'a Object,
    #[serde(serialize_with = "wire")]
    theirs: &'a Object,
}

impl<'a> ConflictRow<'a> {
    pub(super) fn new(c: &'a Conflict) -> ConflictRow<'a> {
        ConflictRow {
            oid: &c.oid,
            remote: &c.remote.0,
            ours: &c.ours,
            theirs: &c.theirs,
        }
    }

    pub(super) fn line(&self) -> String {
        format!(
            "{}  {}  ours: {:?}  theirs: {:?}  (dam resolve {} --ours|--theirs)",
            self.oid.short(),
            self.remote,
            self.ours.base().subject,
            self.theirs.base().subject,
            self.oid.short()
        )
    }
}

#[cfg(test)]
mod tests;
