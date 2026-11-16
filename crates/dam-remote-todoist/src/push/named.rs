use std::collections::HashMap;

use dam_protocol::Mutation;

use super::Failure;
use crate::map::RemoteId;

pub struct ThisPush {
    pub containers_filled: Vec<String>,
    pub created: HashMap<String, String>,
}

impl ThisPush {
    pub fn named_container(&self, m: &Mutation) -> Result<Option<RemoteId>, Failure> {
        let named = match (&m.container_remote_id, &m.container_oid) {
            (Some(remote_id), _) => remote_id,
            (None, Some(oid)) => self.created.get(oid).ok_or_else(|| {
                let short = oid.get(..7).unwrap_or(oid);
                Failure::RefusedWithoutAVerdict(format!(
                    "its parent {short} is not in Todoist yet; run: dam push"
                ))
            })?,
            (None, None) => return Ok(None),
        };
        named.parse().map(Some).map_err(|e| {
            Failure::RefusedWithoutAVerdict(format!("an unusable Todoist container id: {e}"))
        })
    }
}
