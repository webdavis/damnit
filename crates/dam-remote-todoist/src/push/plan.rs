use dam_protocol::{Mutation, WireObject};

use super::Failure;
use super::commands::{command, creating_command, item_args, uuid_for};
use super::shape::{Shape, shape_for};
use crate::map::{Tree, split_remote_id};

/// The Sync API commands one mutation becomes, and what they create.
pub struct Plan {
    pub commands: Vec<serde_json::Value>,
    /// Set when the commands create an object: the shape it takes, so the tree
    /// can take it once Todoist answers with the id it issued.
    pub creates: Option<Shape>,
}

pub fn plan_for(tree: &Tree, m: &Mutation, pending: &[String]) -> Result<Plan, Failure> {
    match m.op.as_str() {
        "delete" => {
            let (kind, id) = addressed(m, "delete")?;
            Ok(Plan {
                commands: vec![command(
                    delete_verb(kind),
                    numbered(m, 0)?,
                    serde_json::json!({ "id": id }),
                )],
                creates: None,
            })
        }
        "create" => {
            let object = m.object.as_ref().ok_or(missing_object("create"))?;
            create_plan(tree, m, object, pending)
        }
        "update" => {
            let object = m.object.as_ref().ok_or(missing_object("update"))?;
            let (kind, id) = addressed(m, "update")?;
            update_plan(tree, m, object, kind, id)
        }
        other => Err(Failure::Mutation(format!("unknown op {other:?}"))),
    }
}

fn create_plan(
    tree: &Tree,
    m: &Mutation,
    object: &WireObject,
    pending: &[String],
) -> Result<Plan, Failure> {
    let shape = shape_for(tree, object, pending).map_err(Failure::Mutation)?;
    // The placeholder is the mutation's own key, so a resend names the same
    // one and Todoist's answer maps it back the same way.
    let temp_id = m.idempotency_key.as_str();
    let mut commands = Vec::new();
    match &shape {
        Shape::Project { parent_id } => {
            let mut args = serde_json::json!({ "name": object.subject });
            if let Some(p) = parent_id {
                args["parent_id"] = p.clone().into();
            }
            commands.push(creating_command(
                "project_add",
                numbered(m, 0)?,
                temp_id,
                args,
            ));
        }
        Shape::Section { project_id } => {
            commands.push(creating_command(
                "section_add",
                numbered(m, 0)?,
                temp_id,
                serde_json::json!({ "name": object.subject, "project_id": project_id }),
            ));
        }
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => {
            let mut args = item_args(object, None);
            args["project_id"] = project_id.clone().into();
            if let Some(s) = section_id {
                args["section_id"] = s.clone().into();
            }
            if let Some(p) = parent_id {
                args["parent_id"] = p.clone().into();
            }
            commands.push(creating_command("item_add", numbered(m, 0)?, temp_id, args));
            if object.task.as_ref().is_some_and(|t| t.done) {
                commands.push(command(
                    "item_close",
                    numbered(m, 1)?,
                    serde_json::json!({ "id": temp_id }),
                ));
            }
        }
    }
    Ok(Plan {
        commands,
        creates: Some(shape),
    })
}

fn update_plan(
    tree: &Tree,
    m: &Mutation,
    object: &WireObject,
    kind: char,
    id: &str,
) -> Result<Plan, Failure> {
    let changed = |f: &str| m.fields.iter().any(|x| x == f);
    let done = object.task.as_ref().is_some_and(|t| t.done);
    let mut commands = Vec::new();
    match kind {
        'p' | 's' => {
            if changed("subject") {
                let verb = if kind == 'p' {
                    "project_update"
                } else {
                    "section_update"
                };
                let n = commands.len();
                commands.push(command(
                    verb,
                    numbered(m, n)?,
                    serde_json::json!({ "id": id, "name": object.subject }),
                ));
            }
            if kind == 'p' && changed("done") {
                let verb = if done {
                    "project_archive"
                } else {
                    "project_unarchive"
                };
                let n = commands.len();
                commands.push(command(
                    verb,
                    numbered(m, n)?,
                    serde_json::json!({ "id": id }),
                ));
            }
        }
        _ => {
            if changed("path") {
                let n = commands.len();
                commands.push(command(
                    "item_move",
                    numbered(m, n)?,
                    move_args(tree, id, object)?,
                ));
            }
            let mut args = item_args(object, Some(&m.fields));
            if args.as_object().is_some_and(|o| !o.is_empty()) {
                args["id"] = id.into();
                let n = commands.len();
                commands.push(command("item_update", numbered(m, n)?, args));
            }
            if changed("done") {
                let verb = if done {
                    "item_close"
                } else {
                    "item_uncomplete"
                };
                let n = commands.len();
                commands.push(command(
                    verb,
                    numbered(m, n)?,
                    serde_json::json!({ "id": id }),
                ));
            }
        }
    }
    Ok(Plan {
        commands,
        creates: None,
    })
}

/// Todoist's move command takes exactly one of project_id, section_id or
/// parent_id; the most specific container the new path resolves to is the one
/// sent.
fn move_args(tree: &Tree, id: &str, object: &WireObject) -> Result<serde_json::Value, Failure> {
    match shape_for(tree, object, &[]).map_err(Failure::Mutation)? {
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => Ok(match (parent_id, section_id) {
            (Some(p), _) => serde_json::json!({ "id": id, "parent_id": p }),
            (None, Some(s)) => serde_json::json!({ "id": id, "section_id": s }),
            (None, None) => serde_json::json!({ "id": id, "project_id": project_id }),
        }),
        _ => Err(Failure::Mutation(
            "a path change would move the task out of task position".into(),
        )),
    }
}

fn numbered(m: &Mutation, n: usize) -> Result<String, Failure> {
    uuid_for(&m.idempotency_key, n).map_err(Failure::Mutation)
}

pub fn missing_object(op: &str) -> Failure {
    Failure::Mutation(format!("{op} without an object"))
}

/// The Todoist object a mutation names, refused by reason when the stored
/// `remote_id` is absent or is not one Todoist could have issued.
fn addressed<'m>(m: &'m Mutation, op: &str) -> Result<(char, &'m str), Failure> {
    let text = m
        .remote_id
        .as_deref()
        .ok_or_else(|| Failure::Mutation(format!("{op} without a Todoist id")))?;
    split_remote_id(text)
        .map_err(|e| Failure::Mutation(format!("{op} with an unusable Todoist id: {e}")))
}

fn delete_verb(kind: char) -> &'static str {
    match kind {
        'p' => "project_delete",
        's' => "section_delete",
        _ => "item_delete",
    }
}
