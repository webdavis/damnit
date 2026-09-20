//! One staged or unstaged change, and one conflict, rendered both ways.

use dam_application::Conflict;
use dam_domain::{Change, Op, changed_fields};

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

pub(crate) fn change_json(change: &Change) -> Result<serde_json::Value, CliError> {
    let before = change.before.as_ref().map(object_json).transpose()?;
    let after = change.after.as_ref().map(object_json).transpose()?;
    Ok(serde_json::json!({
        "oid": change.oid.to_string(),
        "op": match change.op { Op::Create => "create", Op::Update => "update", Op::Delete => "delete" },
        "before": before,
        "after": after,
    }))
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
