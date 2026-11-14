use std::collections::HashMap;

use serde::Deserialize;

use dam_helper::one_line;

#[derive(Debug, Default, Deserialize)]
pub struct SyncResponse {
    #[serde(default)]
    pub sync_token: Option<String>,
    #[serde(default)]
    pub full_sync: bool,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub sections: Vec<Section>,
    #[serde(default)]
    pub items: Vec<Item>,
}

impl SyncResponse {
    pub fn updated_by(self, changes: SyncResponse) -> SyncResponse {
        SyncResponse {
            sync_token: changes.sync_token,
            full_sync: false,
            projects: newest(self.projects, changes.projects, |p| &p.id),
            sections: newest(self.sections, changes.sections, |s| &s.id),
            items: newest(self.items, changes.items, |i| &i.id),
        }
    }
}

fn newest<T>(mut held: Vec<T>, changed: Vec<T>, id: impl Fn(&T) -> &String) -> Vec<T> {
    for change in changed {
        match held.iter().position(|h| id(h) == id(&change)) {
            Some(at) => held[at] = change,
            None => held.push(change),
        }
    }
    held
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
    pub completed_at: Option<String>,
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

#[derive(Debug, Default, Deserialize)]
pub struct SyncWrite {
    #[serde(default, rename = "sync_status")]
    pub verdict_by_uuid: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "temp_id_mapping")]
    pub issued_id_by_temp_id: HashMap<String, String>,
}

const ACCEPTED_VERDICT: &str = "ok";

#[derive(Debug, PartialEq, Eq)]
pub enum NotAccepted {
    Refused(String),
    NoVerdict,
}

impl SyncWrite {
    pub fn accepted(&self, uuid: &str) -> Result<(), NotAccepted> {
        let refused = |why: &str| Err(NotAccepted::Refused(one_line(why)));
        match self.verdict_by_uuid.get(uuid) {
            Some(serde_json::Value::String(word)) if word == ACCEPTED_VERDICT => Ok(()),
            Some(serde_json::Value::Object(error)) => refused(
                error
                    .get("error")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(&serde_json::Value::Object(error.clone()).to_string()),
            ),
            Some(other) => refused(&other.to_string()),
            None => Err(NotAccepted::NoVerdict),
        }
    }
}

#[cfg(test)]
mod tests;
