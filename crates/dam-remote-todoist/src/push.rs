mod carried;
mod commands;
mod plan;
mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, Verdict, WireObject};

use crate::api::{ApiError, Item, Project, Section, SyncWrite, TodoistSync};
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

pub fn push(api: &impl TodoistSync, mutations: Vec<Mutation>) -> Result<PushResponse, ApiError> {
    let sync = api.read_all()?;
    let mut tree = Tree::from_sync(&sync);
    let containers_filled = containers_this_push_fills(&mutations);
    let mut results = Vec::with_capacity(mutations.len());
    for m in &mutations {
        match apply(api, &mut tree, m, &containers_filled) {
            Ok(remote_id) => results.push(MutationResult {
                oid: m.oid.clone(),
                verdict: Verdict::Accepted { remote_id },
            }),
            Err(Failure::OnlyThisMutation(why)) => results.push(MutationResult {
                oid: m.oid.clone(),
                verdict: Verdict::Refused { why },
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
    api: &impl TodoistSync,
    tree: &mut Tree,
    m: &Mutation,
    containers_filled: &[String],
) -> Result<Option<String>, Failure> {
    let plan = plan_for(tree, m, containers_filled)?;
    let written = match plan.commands.is_empty() {
        true => None,
        false => Some(send_every_command(api, &plan.commands)?),
    };
    if let Some(why) = plan.refused {
        return Err(Failure::OnlyThisMutation(why));
    }
    let Some(shape) = plan.creates else {
        return Ok(m.remote_id.clone());
    };
    let object = m.object.as_ref().ok_or(missing_object("create"))?;
    let id = written
        .as_ref()
        .and_then(|w| w.issued_id_by_temp_id.get(resend_stable_temp_id(m)))
        .ok_or_else(|| {
            Failure::OnlyThisMutation("Todoist's answer names no id for the new object".to_string())
        })?
        .clone();
    Ok(Some(add_created_to_tree(tree, object, shape, id)))
}

fn send_every_command(
    api: &impl TodoistSync,
    commands: &[serde_json::Value],
) -> Result<SyncWrite, Failure> {
    let written = api.write(commands)?;
    for sent in commands {
        let uuid = sent["uuid"].as_str().unwrap_or_default();
        written.accepted(uuid).map_err(Failure::OnlyThisMutation)?;
    }
    Ok(written)
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

#[cfg(test)]
mod tests;
