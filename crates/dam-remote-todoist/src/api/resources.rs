//! The Todoist resources a sync returns, and the write commands it takes.
//! Data declaration only: what they mean in dam's terms is the mapper's
//! business.

use std::collections::HashMap;

use serde::Deserialize;

use super::error::bounded;

#[derive(Debug, Deserialize)]
pub struct SyncResponse {
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub inbox_project: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub id: String,
    pub name: String,
    pub project_id: String,
    #[serde(default)]
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub description: String,
    pub project_id: String,
    #[serde(default)]
    pub section_id: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default = "lowest_priority")]
    pub priority: u8,
    #[serde(default)]
    pub due: Option<Due>,
    #[serde(default)]
    pub deadline: Option<Deadline>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub checked: bool,
    #[serde(default)]
    pub is_deleted: bool,
}

fn lowest_priority() -> u8 {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct Due {
    pub date: String,
    #[serde(default)]
    pub is_recurring: bool,
    #[serde(default)]
    pub string: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deadline {
    pub date: String,
}

/// What Todoist answers a write with: its verdict on each command by uuid, and
/// the id it issued for each object a command created, by the placeholder that
/// command used.
#[derive(Debug, Default, Deserialize)]
pub struct SyncWrite {
    #[serde(default)]
    pub sync_status: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub temp_id_mapping: HashMap<String, String>,
}

impl SyncWrite {
    /// Whether Todoist executed the command with this uuid. A verdict is the
    /// string `"ok"` or an error object; anything else, a missing verdict
    /// included, is a failure rather than a silent success.
    pub fn accepted(&self, uuid: &str) -> Result<(), String> {
        match self.sync_status.get(uuid) {
            Some(serde_json::Value::String(word)) if word == "ok" => Ok(()),
            Some(serde_json::Value::Object(error)) => Err(bounded(
                error
                    .get("error")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(&serde_json::Value::Object(error.clone()).to_string()),
            )),
            Some(other) => Err(bounded(&other.to_string())),
            None => Err(format!("Todoist gave no verdict for command {uuid}")),
        }
    }
}
