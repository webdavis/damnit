use dam_protocol::{Mutation, WireObject};

use super::Failure;
use super::commands::{ItemWrite, command, creating_command, item_args, repeatable_command_uuid};
use super::shape::{Shape, shape_for};
use crate::map::{Tree, split_remote_id};

pub struct Plan {
    pub commands: Vec<serde_json::Value>,
    pub creates: Option<Shape>,
}

pub fn resend_stable_temp_id(m: &Mutation) -> &str {
    m.idempotency_key.as_str()
}

pub fn plan_for(
    tree: &Tree,
    m: &Mutation,
    containers_filled_this_push: &[String],
) -> Result<Plan, Failure> {
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
            create_plan(tree, m, object, containers_filled_this_push)
        }
        "update" => {
            let object = m.object.as_ref().ok_or(missing_object("update"))?;
            let (kind, id) = addressed(m, "update")?;
            update_plan(tree, m, object, kind, id)
        }
        other => Err(Failure::OnlyThisMutation(format!("unknown op {other:?}"))),
    }
}

fn create_plan(
    tree: &Tree,
    m: &Mutation,
    object: &WireObject,
    containers_filled_this_push: &[String],
) -> Result<Plan, Failure> {
    let shape =
        shape_for(tree, object, containers_filled_this_push).map_err(Failure::OnlyThisMutation)?;
    let temp_id = resend_stable_temp_id(m);
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
            let mut args = item_args(object, ItemWrite::Create);
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
                    most_specific_container_move_args(tree, id, object)?,
                ));
            }
            let mut args = item_args(object, ItemWrite::Update { changed: &m.fields });
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

fn most_specific_container_move_args(
    tree: &Tree,
    id: &str,
    object: &WireObject,
) -> Result<serde_json::Value, Failure> {
    match shape_for(tree, object, &[]).map_err(Failure::OnlyThisMutation)? {
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => Ok(match (parent_id, section_id) {
            (Some(p), _) => serde_json::json!({ "id": id, "parent_id": p }),
            (None, Some(s)) => serde_json::json!({ "id": id, "section_id": s }),
            (None, None) => serde_json::json!({ "id": id, "project_id": project_id }),
        }),
        _ => Err(Failure::OnlyThisMutation(
            "a path change would move the task out of task position".into(),
        )),
    }
}

fn numbered(m: &Mutation, ordinal: usize) -> Result<String, Failure> {
    repeatable_command_uuid(&m.idempotency_key, ordinal).map_err(Failure::OnlyThisMutation)
}

pub fn missing_object(op: &str) -> Failure {
    Failure::OnlyThisMutation(format!("{op} without an object"))
}

fn addressed<'m>(m: &'m Mutation, op: &str) -> Result<(char, &'m str), Failure> {
    let text = m
        .remote_id
        .as_deref()
        .ok_or_else(|| Failure::OnlyThisMutation(format!("{op} without a Todoist id")))?;
    split_remote_id(text)
        .map_err(|e| Failure::OnlyThisMutation(format!("{op} with an unusable Todoist id: {e}")))
}

fn delete_verb(kind: char) -> &'static str {
    match kind {
        'p' => "project_delete",
        's' => "section_delete",
        _ => "item_delete",
    }
}
