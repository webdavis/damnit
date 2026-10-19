mod error;
mod resources;

pub use error::{ApiError, MAX_ERROR_BODY_CHARS};
pub use resources::{Deadline, Due, Item, Project, Section, SyncResponse, SyncWrite};

use dam_helper::{Secret, base_url, credential};
use error::read_json;

pub const PRODUCTION_BASE: &str = "https://api.todoist.com/api/v1";

#[derive(Debug)]
pub struct TodoistApi {
    agent: ureq::Agent,
    base: String,
    token: Secret,
}

impl TodoistApi {
    pub fn new(base: &str, token: &str) -> TodoistApi {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .timeout_await_100(Some(std::time::Duration::ZERO))
            .max_redirects(0)
            .http_status_as_error(false)
            .build();
        TodoistApi {
            agent: config.into(),
            base: base.trim_end_matches('/').to_string(),
            token: token.into(),
        }
    }

    pub fn from_env(remote: &str) -> Result<TodoistApi, String> {
        let token = credential(remote, "api_token", |variable| std::env::var(variable).ok())?;
        let base = base_url("DAM_TODOIST_BASE_URL")?.unwrap_or_else(|| PRODUCTION_BASE.into());
        Ok(TodoistApi::new(&base, token.expose()))
    }

    pub fn sync_all(&self) -> Result<SyncResponse, ApiError> {
        let body = serde_json::json!({ "sync_token": "*", "resource_types": ["projects", "sections", "items"] });
        let value = self.post("/sync", &body)?;
        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    pub fn sync_commands(&self, commands: &[serde_json::Value]) -> Result<SyncWrite, ApiError> {
        let encoded = serde_json::Value::Array(commands.to_vec()).to_string();
        let response = self
            .agent
            .post(format!("{}/sync", self.base))
            .header("Authorization", &self.authorization())
            .send_form([("commands", encoded.as_str())])
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        let value = read_json(response)?;
        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    fn authorization(&self) -> String {
        format!("Bearer {}", self.token.expose())
    }

    pub fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let response = self
            .agent
            .post(format!("{}{path}", self.base))
            .header("Authorization", &self.authorization())
            .send_json(body)
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        read_json(response)
    }
}

#[cfg(test)]
mod tests;
