use std::collections::HashMap;

use serde::Deserialize;

use super::error::one_short_line;

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

#[derive(Debug, Default, Deserialize)]
pub struct SyncWrite {
    #[serde(default, rename = "sync_status")]
    pub verdict_by_uuid: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "temp_id_mapping")]
    pub issued_id_by_temp_id: HashMap<String, String>,
}

const ACCEPTED_VERDICT: &str = "ok";

impl SyncWrite {
    pub fn accepted(&self, uuid: &str) -> Result<(), String> {
        match self.verdict_by_uuid.get(uuid) {
            Some(serde_json::Value::String(word)) if word == ACCEPTED_VERDICT => Ok(()),
            Some(serde_json::Value::Object(error)) => Err(one_short_line(
                error
                    .get("error")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(&serde_json::Value::Object(error.clone()).to_string()),
            )),
            Some(other) => Err(one_short_line(&other.to_string())),
            None => Err(format!("Todoist gave no verdict for command {uuid}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verdict_that_is_neither_ok_nor_an_error_object_is_a_failure() {
        let written = SyncWrite {
            verdict_by_uuid: HashMap::from([
                ("u-0".to_string(), serde_json::json!(true)),
                ("u-1".to_string(), serde_json::json!("done")),
            ]),
            ..SyncWrite::default()
        };
        assert!(written.accepted("u-0").is_err());
        assert!(written.accepted("u-1").is_err());
    }
}
