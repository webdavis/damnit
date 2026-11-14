mod carried;
mod commands;
mod created;
mod plan;
mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, Verdict};

use crate::api::{ApiError, NotAccepted, SyncWrite, TodoistSync};
use crate::map::Tree;
use commands::Command;
use created::add_created_to_tree;
use plan::{TreeChange, missing_object, plan_for, resend_stable_temp_id};

enum Failure {
    OnlyThisMutation(String),
    Unanswered,
    StopsThePush(ApiError),
}

const FIRST_SERVER_ERROR: u16 = 500;

impl From<ApiError> for Failure {
    fn from(e: ApiError) -> Failure {
        match e {
            rate @ ApiError::RateLimited { .. } => Failure::StopsThePush(rate),
            ApiError::Http { status, .. } if status < FIRST_SERVER_ERROR => {
                Failure::OnlyThisMutation(e.to_string())
            }
            ApiError::Http { .. }
            | ApiError::Transport { .. }
            | ApiError::Decode { .. }
            | ApiError::TooLarge { .. } => Failure::Unanswered,
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
            Err(Failure::Unanswered) => break,
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
    let shape = match plan.changes {
        TreeChange::Creates(shape) => shape,
        TreeChange::MovesItem { id, to } => {
            let (section, parent) = (to.section_id.as_deref(), to.parent_id.as_deref());
            tree.move_item(&id, &to.project_id, section, parent);
            return Ok(m.remote_id.clone());
        }
        TreeChange::Removes(gone) => {
            tree.remove(&gone);
            return Ok(m.remote_id.clone());
        }
        TreeChange::Nothing => return Ok(m.remote_id.clone()),
    };
    let object = m.object.as_ref().ok_or(missing_object("create"))?;
    let id = written
        .as_ref()
        .and_then(|w| w.issued_id_by_temp_id.get(resend_stable_temp_id(m)))
        .ok_or(Failure::Unanswered)?
        .clone();
    Ok(Some(add_created_to_tree(tree, object, shape, id)))
}

fn send_every_command(api: &impl TodoistSync, commands: &[Command]) -> Result<SyncWrite, Failure> {
    let encoded = commands
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Failure::OnlyThisMutation(e.to_string()))?;
    let written = api.write(&encoded)?;
    let not_accepted: Vec<NotAccepted> = commands
        .iter()
        .filter_map(|sent| written.accepted(&sent.uuid).err())
        .collect();
    let todoist_refused_every_command = not_accepted.len() == commands.len()
        && not_accepted
            .iter()
            .all(|n| matches!(n, NotAccepted::Refused(_)));
    match not_accepted.into_iter().next() {
        None => Ok(written),
        Some(NotAccepted::Refused(why)) if todoist_refused_every_command => {
            Err(Failure::OnlyThisMutation(why))
        }
        Some(_) => Err(Failure::Unanswered),
    }
}

#[cfg(test)]
mod tests;
