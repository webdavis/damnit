mod resources;

pub use dam_helper::{MAX_ERROR_BODY_CHARS, ServiceError as ApiError};
pub use resources::{Deadline, Due, Item, Project, Section, SyncResponse, SyncWrite};

use dam_helper::{Secret, agent, base_url, credential, read_answer};

pub const PRODUCTION_BASE: &str = "https://api.todoist.com/api/v1";

const SERVICE: &str = "Todoist";

pub trait TodoistSync {
    fn read_all(&self) -> Result<SyncResponse, ApiError>;
    fn write(&self, commands: &[serde_json::Value]) -> Result<SyncWrite, ApiError>;
}

#[derive(Debug)]
pub struct TodoistApi {
    agent: ureq::Agent,
    base: String,
    token: Secret,
}

impl TodoistApi {
    pub fn new(base: &str, token: &str) -> TodoistApi {
        TodoistApi {
            agent: agent(),
            base: base.trim_end_matches('/').to_string(),
            token: token.into(),
        }
    }

    pub fn from_env(remote: &str) -> Result<TodoistApi, String> {
        let token = credential(remote, "api_token", |variable| std::env::var(variable).ok())?;
        let base = base_url("DAM_TODOIST_BASE_URL")?.unwrap_or_else(|| PRODUCTION_BASE.into());
        Ok(TodoistApi::new(&base, token.expose()))
    }

    fn authorization(&self) -> String {
        format!("Bearer {}", self.token.expose())
    }

    fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, ApiError> {
        let response = self
            .agent
            .post(format!("{}{path}", self.base))
            .header("Authorization", &self.authorization())
            .send_json(body)
            .map_err(|e| ApiError::transport(SERVICE, e))?;
        json(response)
    }
}

impl TodoistSync for TodoistApi {
    fn read_all(&self) -> Result<SyncResponse, ApiError> {
        let body = serde_json::json!({ "sync_token": "*", "resource_types": ["projects", "sections", "items"] });
        let value = self.post("/sync", &body)?;
        serde_json::from_value(value).map_err(|e| ApiError::decode(SERVICE, e))
    }

    fn write(&self, commands: &[serde_json::Value]) -> Result<SyncWrite, ApiError> {
        let encoded = serde_json::Value::Array(commands.to_vec()).to_string();
        let response = self
            .agent
            .post(format!("{}/sync", self.base))
            .header("Authorization", &self.authorization())
            .send_form([("commands", encoded.as_str())])
            .map_err(|e| ApiError::transport(SERVICE, e))?;
        let value = json(response)?;
        serde_json::from_value(value).map_err(|e| ApiError::decode(SERVICE, e))
    }
}

fn json(response: ureq::http::Response<ureq::Body>) -> Result<serde_json::Value, ApiError> {
    let text = read_answer(SERVICE, response)?;
    if text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| ApiError::decode(SERVICE, e))
}

#[cfg(test)]
pub(crate) mod fake;
#[cfg(test)]
mod tests;
