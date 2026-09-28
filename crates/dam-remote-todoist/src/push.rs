mod commands;
mod plan;
mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, WireObject};

use crate::api::{ApiError, Item, Project, Section, SyncWrite, TodoistApi};
use crate::map::{Tree, api_priority, remote_id};
use plan::{missing_object, plan_for, resend_stable_temp_id};
use shape::Shape;

enum Failure {
    OnlyThisMutation(String),
    StopsThePush(ApiError),
}

impl From<ApiError> for Failure {
    fn from(e: ApiError) -> Failure {
        match e {
            rate @ ApiError::RateLimited { .. } => Failure::StopsThePush(rate),
            other => Failure::OnlyThisMutation(other.to_string()),
        }
    }
}

impl From<String> for Failure {
    fn from(why: String) -> Failure {
        Failure::OnlyThisMutation(why)
    }
}

pub fn push(api: &TodoistApi, mutations: Vec<Mutation>) -> Result<PushResponse, ApiError> {
    let sync = api.sync_all()?;
    let mut tree = Tree::from_sync(&sync);
    let containers_filled = containers_this_push_fills(&mutations);
    let mut results = Vec::with_capacity(mutations.len());
    for m in &mutations {
        match apply(api, &mut tree, m, &containers_filled) {
            Ok(remote_id) => results.push(MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id,
                why: None,
            }),
            Err(Failure::OnlyThisMutation(why)) => results.push(MutationResult {
                oid: m.oid.clone(),
                ok: false,
                remote_id: m.remote_id.clone(),
                why: Some(why),
            }),
            Err(Failure::StopsThePush(e)) => return Err(e),
        }
    }
    Ok(PushResponse { results })
}

fn containers_this_push_fills(mutations: &[Mutation]) -> Vec<String> {
    mutations
        .iter()
        .filter_map(|m| m.object.as_ref())
        .map(|o| o.path.clone())
        .collect()
}

fn apply(
    api: &TodoistApi,
    tree: &mut Tree,
    m: &Mutation,
    containers_filled: &[String],
) -> Result<Option<String>, Failure> {
    let plan = plan_for(tree, m, containers_filled)?;
    if plan.commands.is_empty() {
        return Ok(m.remote_id.clone());
    }
    let written = api.sync_commands(&plan.commands)?;
    require_every_command_accepted(&written, &plan.commands)?;
    let Some(shape) = plan.creates else {
        return Ok(m.remote_id.clone());
    };
    let object = m.object.as_ref().ok_or(missing_object("create"))?;
    let id = written
        .issued_id_by_temp_id
        .get(resend_stable_temp_id(m))
        .ok_or_else(|| {
            Failure::OnlyThisMutation("Todoist's answer names no id for the new object".to_string())
        })?
        .clone();
    Ok(Some(add_created_to_tree(tree, object, shape, id)))
}

fn require_every_command_accepted(
    written: &SyncWrite,
    commands: &[serde_json::Value],
) -> Result<(), Failure> {
    for sent in commands {
        let uuid = sent["uuid"].as_str().unwrap_or_default();
        written.accepted(uuid).map_err(Failure::OnlyThisMutation)?;
    }
    Ok(())
}

fn add_created_to_tree(tree: &mut Tree, object: &WireObject, shape: Shape, id: String) -> String {
    match shape {
        Shape::Project { parent_id } => {
            tree.add_project(Project {
                id: id.clone(),
                name: object.subject.clone(),
                parent_id,
                is_deleted: false,
                is_archived: false,
                inbox_project: false,
            });
            remote_id('p', &id)
        }
        Shape::Section { project_id } => {
            tree.add_section(Section {
                id: id.clone(),
                name: object.subject.clone(),
                project_id,
                is_deleted: false,
            });
            remote_id('s', &id)
        }
        Shape::Item {
            project_id,
            section_id,
            parent_id,
        } => {
            let task = object.task.as_ref();
            tree.add_item(Item {
                id: id.clone(),
                content: object.subject.clone(),
                description: object.body.clone(),
                project_id,
                section_id,
                parent_id,
                priority: task.map(|t| api_priority(t.priority)).unwrap_or(1),
                due: None,
                deadline: None,
                labels: object.labels.clone(),
                checked: task.is_some_and(|t| t.done),
                is_deleted: false,
            });
            remote_id('i', &id)
        }
    }
}
