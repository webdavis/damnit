mod commands;
mod plan;
mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, WireObject};

use crate::api::{ApiError, Item, Project, Section, TodoistApi};
use crate::map::{Tree, api_priority, remote_id};
use plan::{missing_object, plan_for};
use shape::Shape;

/// Why one mutation did not happen.
enum Failure {
    /// Reported against that mutation; the rest of the push goes on.
    Mutation(String),
    /// The push stops here. Attempting the rest would report failures Todoist
    /// never saw, and dam would mark the batch pushed on the strength of them.
    Halt(String),
}

impl From<ApiError> for Failure {
    fn from(e: ApiError) -> Failure {
        match e {
            rate @ ApiError::RateLimited { .. } => Failure::Halt(rate.to_string()),
            other => Failure::Mutation(other.to_string()),
        }
    }
}

impl From<String> for Failure {
    fn from(why: String) -> Failure {
        Failure::Mutation(why)
    }
}

pub fn push(api: &TodoistApi, mutations: Vec<Mutation>) -> Result<PushResponse, String> {
    let sync = api.sync_all().map_err(|e| e.to_string())?;
    let mut tree = Tree::from_sync(&sync);
    // Each object's own `path` names the container it lives directly under; collecting
    // those (not each object's own subject-appended location) is what lets a depth-1
    // create become a section when one of its children arrives later in the same push.
    let pending: Vec<String> = mutations
        .iter()
        .filter_map(|m| m.object.as_ref())
        .map(|o| o.path.clone())
        .collect();
    let mut results = Vec::with_capacity(mutations.len());
    for m in &mutations {
        match apply(api, &mut tree, m, &pending) {
            Ok(remote_id) => results.push(MutationResult {
                oid: m.oid.clone(),
                ok: true,
                remote_id,
                why: None,
            }),
            Err(Failure::Mutation(why)) => results.push(MutationResult {
                oid: m.oid.clone(),
                ok: false,
                remote_id: m.remote_id.clone(),
                why: Some(why),
            }),
            Err(Failure::Halt(why)) => return Err(why),
        }
    }
    Ok(PushResponse { results })
}

/// Sends one mutation's commands as a single sync write and reads Todoist's
/// verdict on each of them. Every command has to be accepted for the mutation
/// to be: a mutation that half happened is one dam must send again.
fn apply(
    api: &TodoistApi,
    tree: &mut Tree,
    m: &Mutation,
    pending: &[String],
) -> Result<Option<String>, Failure> {
    let plan = plan_for(tree, m, pending)?;
    if plan.commands.is_empty() {
        return Ok(m.remote_id.clone());
    }
    let written = api.sync_commands(&plan.commands)?;
    for sent in &plan.commands {
        let uuid = sent["uuid"].as_str().unwrap_or_default();
        written.accepted(uuid).map_err(Failure::Mutation)?;
    }
    let Some(shape) = plan.creates else {
        return Ok(m.remote_id.clone());
    };
    let object = m.object.as_ref().ok_or(missing_object("create"))?;
    let id = written
        .temp_id_mapping
        .get(&m.idempotency_key)
        .ok_or_else(|| {
            Failure::Mutation("Todoist's answer names no id for the new object".to_string())
        })?
        .clone();
    Ok(Some(take_created(tree, object, shape, id)))
}

/// Puts a newly created object into the local picture of the tree and names
/// the remote id dam should store for it.
fn take_created(tree: &mut Tree, object: &WireObject, shape: Shape, id: String) -> String {
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
