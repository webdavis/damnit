mod removal;
mod update;

use dam_protocol::{Mutation, WireObject};

use super::Failure;
use super::commands::{
    Command, ItemWrite, args, command, creating_command, item_args, repeatable_command_uuid,
};
use super::named::ThisPush;
use super::shape::{ItemPlace, Shape, shape_for, shape_inside};
use super::weekday_of_month::in_todoist_words;
use crate::map::{RemoteId, Tree};

pub struct Plan {
    pub commands: Vec<Command>,
    pub changes: Vec<TreeChange>,
    pub refused: Option<String>,
}

pub enum TreeChange {
    Creates(Shape),
    MovesItem { id: String, to: ItemPlace },
    Renames { renamed: RemoteId, name: String },
    Removes(RemoteId),
}

pub fn resend_stable_temp_id(m: &Mutation) -> &str {
    m.idempotency_key.as_str()
}

pub fn plan_for(tree: &Tree, m: &Mutation, this_push: &ThisPush) -> Result<Plan, Failure> {
    match m.op.as_str() {
        "delete" => removal::removal_plan(tree, m, addressed(m, "delete")?),
        "create" => {
            let object = m.object.as_ref().ok_or(missing_object("create"))?;
            let object = &in_todoist_words(m, object)?;
            create_plan(tree, m, object, this_push)
        }
        "update" => {
            let object = m.object.as_ref().ok_or(missing_object("update"))?;
            let object = &in_todoist_words(m, object)?;
            let remote = addressed(m, "update")?;
            update::update_plan(tree, m, object, remote, this_push)
        }
        other => Err(Failure::RefusedWithoutAVerdict(format!(
            "unknown op {other:?}"
        ))),
    }
}

fn create_plan(
    tree: &Tree,
    m: &Mutation,
    object: &WireObject,
    this_push: &ThisPush,
) -> Result<Plan, Failure> {
    let filled = &this_push.containers_filled;
    let shape = match this_push.named_container(m)? {
        Some(container) => shape_inside(tree, object, filled, &container),
        None => shape_for(tree, object, filled),
    }
    .map_err(Failure::RefusedWithoutAVerdict)?;
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
        Shape::Item(ItemPlace {
            project_id,
            section_id,
            parent_id,
        }) => {
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
        changes: vec![TreeChange::Creates(shape)],
        refused: None,
    })
}

fn numbered(m: &Mutation, ordinal: usize) -> Result<String, Failure> {
    repeatable_command_uuid(&m.idempotency_key, ordinal).map_err(Failure::RefusedWithoutAVerdict)
}

pub fn missing_object(op: &str) -> Failure {
    Failure::RefusedWithoutAVerdict(format!("{op} without an object"))
}

fn addressed(m: &Mutation, op: &str) -> Result<RemoteId, Failure> {
    let text = m
        .remote_id
        .as_deref()
        .ok_or_else(|| Failure::RefusedWithoutAVerdict(format!("{op} without a Todoist id")))?;
    text.parse().map_err(|e| {
        Failure::RefusedWithoutAVerdict(format!("{op} with an unusable Todoist id: {e}"))
    })
}
