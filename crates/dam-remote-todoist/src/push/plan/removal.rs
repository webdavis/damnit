use dam_protocol::Mutation;

use super::{Failure, Plan, TreeChange, numbered};
use crate::map::{RemoteId, Tree};
use crate::push::commands::{args, command};

pub fn removal_plan(tree: &Tree, m: &Mutation, remote: RemoteId) -> Result<Plan, Failure> {
    if let Some(held) = tree.what_it_still_holds(&remote) {
        return Err(Failure::OnlyThisMutation(format!(
            "Todoist still shows {held} under it, and removing it would remove them too; \
             pull, then push"
        )));
    }
    let (verb, id) = match &remote {
        RemoteId::Project(id) => ("project_delete", id.clone()),
        RemoteId::Section(id) => ("section_delete", id.clone()),
        RemoteId::Item(id) => ("item_delete", id.clone()),
    };
    Ok(Plan {
        commands: vec![command(verb, numbered(m, 0)?, args([("id", id.into())]))],
        changes: vec![TreeChange::Removes(remote)],
        refused: None,
    })
}
