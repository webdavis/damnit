use std::collections::BTreeMap;

use dam_application::StoreError;
use dam_domain::{Channel, Details};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Row {
    meta: BTreeMap<String, String>,
    host: Option<String>,
    dir: Option<String>,
    repo: Option<String>,
    branch: Option<String>,
    at: String,
    channel: String,
}

pub(super) fn details_to_json(details: Option<&Details>) -> Result<Option<String>, StoreError> {
    details
        .map(|d| {
            serde_json::to_string(&Row {
                meta: d.meta.clone(),
                host: d.host.clone(),
                dir: d.dir.clone(),
                repo: d.repo.clone(),
                branch: d.branch.clone(),
                at: d.at.to_string(),
                channel: d.channel.as_str().to_string(),
            })
            .map_err(|e| StoreError::Failed(format!("details: {e}")))
        })
        .transpose()
}

pub(super) fn details_from_json(json: Option<String>) -> Result<Option<Details>, StoreError> {
    let Some(json) = json else {
        return Ok(None);
    };
    let failed = |what: String| StoreError::Failed(format!("stored details: {what}"));
    let row: Row = serde_json::from_str(&json).map_err(|e| failed(e.to_string()))?;
    Ok(Some(Details {
        meta: row.meta,
        host: row.host,
        dir: row.dir,
        repo: row.repo,
        branch: row.branch,
        at: row
            .at
            .parse()
            .map_err(|e: jiff::Error| failed(e.to_string()))?,
        channel: Channel::parse(&row.channel).ok_or_else(|| failed(row.channel.clone()))?,
    }))
}
