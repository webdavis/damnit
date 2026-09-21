//! One staged or unstaged change, and one conflict, rendered both ways.

use dam_application::Conflict;
use dam_domain::{Change, Field, Object, Op, When, changed_fields, touched_fields};

use crate::error::CliError;
use crate::output::object_json;

pub(crate) fn change_line(change: &Change) -> String {
    let subject = change
        .after
        .as_ref()
        .or(change.before.as_ref())
        .map(|o| o.base().subject.as_str())
        .unwrap_or("");
    match change.op {
        Op::Create => format!("new      {}  {subject}", change.oid.short()),
        Op::Delete => format!("removed  {}  {subject}", change.oid.short()),
        Op::Update => {
            let fields = match (&change.before, &change.after) {
                (Some(b), Some(a)) => changed_fields(b, a)
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            };
            format!("changed  {}  {subject}  ({fields})", change.oid.short())
        }
    }
}

/// One change as a client reads it: what moved, and enough of the resulting
/// object to render a row. `full` embeds the whole before and after object
/// beside that, which is what a client wants only when it needs every field.
pub(crate) fn change_json(change: &Change, full: bool) -> Result<serde_json::Value, CliError> {
    let fields: Vec<&str> = touched_fields(change).iter().map(Field::as_str).collect();
    let mut row = serde_json::Map::new();
    row.insert("oid".into(), change.oid.to_string().into());
    row.insert("op".into(), op_name(change.op).into());
    row.insert("fields".into(), fields.into());
    row.extend(state_json(change));
    if full {
        let before = change.before.as_ref().map(object_json).transpose()?;
        let after = change.after.as_ref().map(object_json).transpose()?;
        row.insert("before".into(), before.into());
        row.insert("after".into(), after.into());
    }
    Ok(serde_json::Value::Object(row))
}

fn op_name(op: Op) -> &'static str {
    match op {
        Op::Create => "create",
        Op::Update => "update",
        Op::Delete => "delete",
    }
}

/// The fields a client shows in a list, read off the state the change left
/// behind. A delete leaves none, so its row describes the object it removed.
fn state_json(change: &Change) -> serde_json::Map<String, serde_json::Value> {
    let mut state = serde_json::Map::new();
    let Some(object) = change.after.as_ref().or(change.before.as_ref()) else {
        return state;
    };
    let base = object.base();
    state.insert("kind".into(), object.kind().as_str().into());
    state.insert("subject".into(), base.subject.clone().into());
    state.insert("path".into(), base.path.as_str().into());
    state.insert("labels".into(), base.labels.iter().cloned().collect());
    match object {
        Object::Task(t) => {
            state.insert("done".into(), t.done.into());
            state.insert(
                "completed_at".into(),
                t.completed_at.map(|at| at.to_string()).into(),
            );
            state.insert("priority".into(), t.priority.get().into());
            state.insert("due".into(), t.due.as_ref().map(When::to_text).into());
        }
        Object::Event(e) => {
            state.insert("start".into(), e.start.to_text().into());
            state.insert("end".into(), e.end.to_text().into());
        }
    }
    state
}

pub(super) fn conflict_line(c: &Conflict) -> String {
    format!(
        "{}  {}  ours: {:?}  theirs: {:?}  (dam resolve {} --ours|--theirs)",
        c.oid.short(),
        c.remote.0,
        c.ours.base().subject,
        c.theirs.base().subject,
        c.oid.short()
    )
}

pub(super) fn conflict_json(c: &Conflict) -> Result<serde_json::Value, CliError> {
    Ok(serde_json::json!({
        "oid": c.oid.to_string(),
        "remote": c.remote.0,
        "ours": object_json(&c.ours)?,
        "theirs": object_json(&c.theirs)?,
    }))
}

#[cfg(test)]
mod tests;
