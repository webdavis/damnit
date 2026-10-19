use dam_protocol::PullResponse;

use crate::api::{ApiError, TodoistSync};
use crate::map::{RemoteId, Tree, item_to_wire, project_to_wire, section_to_wire};

pub fn pull(api: &impl TodoistSync) -> Result<PullResponse, ApiError> {
    let sync = api.read_all()?;
    let tree = Tree::from_sync(&sync);
    let mut objects = Vec::new();
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
        }
    }
    Ok(PullResponse {
        objects,
        removed,
        ..PullResponse::default()
    })
}

#[cfg(test)]
mod tests;
