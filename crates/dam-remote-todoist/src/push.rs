mod carried;
mod commands;
mod created;
mod failure;
mod off_limits;
mod plan;
mod shape;

use dam_protocol::{Mutation, MutationResult, PushResponse, Verdict};

use crate::api::{ApiError, SyncWrite, TodoistSync};
use crate::map::Tree;
use commands::Command;
use created::add_created_to_tree;
use failure::{Failure, every_command_accepted, no_id_named_for};
use off_limits::OffLimits;
use plan::{TreeChange, missing_object, plan_for, resend_stable_temp_id};

pub fn push(api: &impl TodoistSync, mutations: Vec<Mutation>) -> Result<PushResponse, ApiError> {
    let sync = api.read_all()?;
    let mut tree = Tree::from_sync(&sync);
    let containers_filled = containers_this_push_fills(&mutations);
    let mut results = Vec::with_capacity(mutations.len());
    let mut off_limits = OffLimits::default();
    let mut stopped = None;
    for m in &mutations {
        let verdict = match off_limits.why_left_out(m) {
            Some(why) => Verdict::Unanswered { why },
            None => match apply(api, &mut tree, m, &containers_filled) {
                Ok(remote_id) => Verdict::Accepted { remote_id },
                Err(Failure::OnlyThisMutation(why)) => Verdict::Refused { why },
                Err(Failure::Unanswered(why)) => Verdict::Unanswered { why },
                Err(Failure::Unreachable(why)) => {
                    off_limits.end_the_call(&why);
                    Verdict::Unanswered { why }
                }
                Err(Failure::StopsThePush(why)) => {
                    off_limits.end_the_call(&why);
                    stopped = Some(why.clone());
                    Verdict::Unanswered { why }
                }
            },
        };
        if matches!(verdict, Verdict::Unanswered { .. }) {
            off_limits.unanswered(m);
        }
        results.push(MutationResult {
            oid: m.oid.clone(),
            verdict,
        });
    }
    Ok(PushResponse { results, stopped })
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
        .ok_or_else(|| no_id_named_for(&shape))?
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
    every_command_accepted(commands, &written)?;
    Ok(written)
}

#[cfg(test)]
mod tests;
