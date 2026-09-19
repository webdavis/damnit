use std::fmt;

use serde::Deserialize;

pub const PRODUCTION_BASE: &str = "https://api.todoist.com/api/v1";

#[derive(Debug)]
pub enum ApiError {
    Http { status: u16, body: String },
    Transport(String),
    Decode(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Http { status, body } => write!(f, "Todoist answered {status}: {body}"),
            ApiError::Transport(s) => write!(f, "reaching Todoist: {s}"),
            ApiError::Decode(s) => write!(f, "reading Todoist's answer: {s}"),
        }
    }
}

impl std::error::Error for ApiError {}

pub struct TodoistApi {
    agent: ureq::Agent,
    base: String,
    token: String,
}

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

impl TodoistApi {
    pub fn new(base: &str, token: &str) -> TodoistApi {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .timeout_await_100(Some(std::time::Duration::ZERO))
            .http_status_as_error(false)
            .build();
        TodoistApi {
            agent: config.into(),
            base: base.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    pub fn from_env() -> Result<TodoistApi, String> {
        let token = std::env::var("DAM_TODOIST_API_TOKEN")
            .ok()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                "DAM_TODOIST_API_TOKEN is not set; declare api_token under [remote.todoist]"
                    .to_string()
            })?;
        let base =
            std::env::var("DAM_TODOIST_BASE_URL").unwrap_or_else(|_| PRODUCTION_BASE.to_string());
        Ok(TodoistApi::new(&base, &token))
    }

    pub fn sync_all(&self) -> Result<SyncResponse, ApiError> {
        let body = serde_json::json!({ "sync_token": "*", "resource_types": ["projects", "sections", "items"] });
        let value = self.post("/sync", &body)?;
        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let response = self
            .agent
            .post(format!("{}{path}", self.base))
            .header("Authorization", &format!("Bearer {}", self.token))
            .send_json(body)
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        read_json(response)
    }

    pub fn delete(&self, path: &str) -> Result<(), ApiError> {
        let response = self
            .agent
            .delete(format!("{}{path}", self.base))
            .header("Authorization", &format!("Bearer {}", self.token))
            .call()
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        read_json(response).map(|_| ())
    }
}

fn read_json(response: ureq::http::Response<ureq::Body>) -> Result<serde_json::Value, ApiError> {
    let status = response.status().as_u16();
    let text = response
        .into_body()
        .read_to_string()
        .map_err(|e| ApiError::Transport(e.to_string()))?;
    if !(200..300).contains(&status) {
        return Err(ApiError::Http { status, body: text });
    }
    if text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| ApiError::Decode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_never_includes_the_token() {
        let http = ApiError::Http {
            status: 404,
            body: "not found".into(),
        };
        assert!(!http.to_string().contains("sentinel-token"));
        let transport = ApiError::Transport("connection refused".into());
        assert!(!transport.to_string().contains("sentinel-token"));
    }

    #[test]
    fn new_trims_a_trailing_slash_from_the_base() {
        let api = TodoistApi::new("http://example.test/", "tok");
        assert_eq!(api.base, "http://example.test");
    }
}
