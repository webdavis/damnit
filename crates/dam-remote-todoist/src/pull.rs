use dam_protocol::PullResponse;

use crate::api::TodoistApi;
use crate::map::{Tree, item_to_wire, project_to_wire, remote_id, section_to_wire};

pub fn pull(api: &TodoistApi) -> Result<PullResponse, String> {
    let sync = api.sync_all().map_err(|e| e.to_string())?;
    let tree = Tree::from_sync(&sync);
    let mut objects = Vec::new();
    let mut removed = Vec::new();
    for p in &sync.projects {
        if p.is_deleted {
            removed.push(remote_id('p', &p.id));
        } else {
            objects.push(project_to_wire(&tree, p));
        }
    }
    for s in &sync.sections {
        if s.is_deleted {
            removed.push(remote_id('s', &s.id));
        } else {
            objects.push(section_to_wire(&tree, s));
        }
    }
    for i in &sync.items {
        if i.is_deleted {
            removed.push(remote_id('i', &i.id));
        } else if let Some(w) = item_to_wire(&tree, i) {
            objects.push(w);
        }
    }
    Ok(PullResponse {
        objects,
        removed,
        sync: None,
    })
}
