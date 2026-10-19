mod update;

use dam_protocol::{Mutation, WireObject};

use super::Failure;
use super::commands::{
    Command, ItemWrite, args, command, creating_command, item_args, repeatable_command_uuid,
};
use super::shape::{Shape, shape_for};
use crate::map::{RemoteId, Tree};

pub struct Plan {
    pub commands: Vec<Command>,
    pub creates: Option<Shape>,
    pub refused: Option<String>,
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
            let (verb, id) = delete_verb(addressed(m, "delete")?);
            Ok(Plan {
                commands: vec![command(verb, numbered(m, 0)?, args([("id", id.into())]))],
                creates: None,
                refused: None,
            })
        }
        "create" => {
            let object = m.object.as_ref().ok_or(missing_object("create"))?;
            create_plan(tree, m, object, containers_filled_this_push)
        }
        "update" => {
            let object = m.object.as_ref().ok_or(missing_object("update"))?;
            let remote = addressed(m, "update")?;
            update::update_plan(tree, m, object, remote)
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
            let mut project = args([("name", object.subject.clone().into())]);
            if let Some(p) = parent_id {
                project.insert("parent_id".into(), p.clone().into());
            }
            commands.push(creating_command(
                "project_add",
                numbered(m, 0)?,
                temp_id,
                project,
            ));
        }
        Shape::Section { project_id } => {
            commands.push(creating_command(
                "section_add",
                numbered(m, 0)?,
                temp_id,
                args([
                    ("name", object.subject.clone().into()),
                    ("project_id", project_id.clone().into()),
                ]),
            ));
        }
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => {
            let mut item = item_args(object, ItemWrite::Create);
            item.insert("project_id".into(), project_id.clone().into());
            if let Some(s) = section_id {
                item.insert("section_id".into(), s.clone().into());
            }
            if let Some(p) = parent_id {
                item.insert("parent_id".into(), p.clone().into());
            }
            commands.push(creating_command("item_add", numbered(m, 0)?, temp_id, item));
            if object.task.as_ref().is_some_and(|t| t.done) {
                commands.push(command(
                    "item_close",
                    numbered(m, 1)?,
                    args([("id", temp_id.into())]),
                ));
            }
        }
    }
    Ok(Plan {
        commands,
        creates: Some(shape),
        refused: None,
    })
}

fn numbered(m: &Mutation, ordinal: usize) -> Result<String, Failure> {
    repeatable_command_uuid(&m.idempotency_key, ordinal).map_err(Failure::OnlyThisMutation)
}

pub fn missing_object(op: &str) -> Failure {
    Failure::OnlyThisMutation(format!("{op} without an object"))
}

fn addressed(m: &Mutation, op: &str) -> Result<RemoteId, Failure> {
    let text = m
        .remote_id
        .as_deref()
        .ok_or_else(|| Failure::OnlyThisMutation(format!("{op} without a Todoist id")))?;
    text.parse()
        .map_err(|e| Failure::OnlyThisMutation(format!("{op} with an unusable Todoist id: {e}")))
}

fn delete_verb(remote: RemoteId) -> (&'static str, String) {
    match remote {
        RemoteId::Project(id) => ("project_delete", id),
        RemoteId::Section(id) => ("section_delete", id),
        RemoteId::Item(id) => ("item_delete", id),
    }
}
