use dam_protocol::{PullResponse, RejectedObject};

use crate::api::{ApiError, TodoistSync};
use crate::map::{RemoteId, Tree, item_to_wire, project_to_wire, section_to_wire};

const UNPLACEABLE: &str = "its parent chain loops or its project or section is missing";

pub fn pull(api: &impl TodoistSync) -> Result<PullResponse, ApiError> {
    let sync = api.read_all()?;
    let tree = Tree::from_sync(&sync);
    let mut objects = Vec::new();
    let mut rejected = Vec::new();
    let mut removed = Vec::new();
    for p in &sync.projects {
        if p.is_deleted {
            removed.push(RemoteId::Project(p.id.clone()).to_string());
        } else {
            objects.push(project_to_wire(&tree, p));
        }
    }
    for s in &sync.sections {
        if s.is_deleted {
            removed.push(RemoteId::Section(s.id.clone()).to_string());
        } else {
            objects.push(section_to_wire(&tree, s));
        }
    }
    for i in &sync.items {
        if i.is_deleted {
            removed.push(RemoteId::Item(i.id.clone()).to_string());
        } else if let Some(w) = item_to_wire(&tree, i) {
            objects.push(w);
        } else {
            rejected.push(RejectedObject {
                remote_id: RemoteId::Item(i.id.clone()).to_string(),
                why: UNPLACEABLE.into(),
            });
        }
    }
    Ok(PullResponse {
        objects,
        rejected,
        removed,
        ..PullResponse::default()
    })
}

#[cfg(test)]
mod tests;
