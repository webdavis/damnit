//! The Todoist HTTP client: one sync read and one batch of write commands.

mod error;
mod resources;
mod token;

pub use error::{ApiError, MAX_ERROR_BODY};
pub use resources::{Deadline, Due, Item, Project, Section, SyncResponse, SyncWrite};
pub use token::ApiToken;

use error::read_json;

pub const PRODUCTION_BASE: &str = "https://api.todoist.com/api/v1";

#[derive(Debug)]
pub struct TodoistApi {
    agent: ureq::Agent,
    base: String,
    token: ApiToken,
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
            token: token.into(),
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
        let base = base_from(std::env::var("DAM_TODOIST_BASE_URL"))?;
        Ok(TodoistApi::new(&base, &token))
    }

    pub fn sync_all(&self) -> Result<SyncResponse, ApiError> {
        let body = serde_json::json!({ "sync_token": "*", "resource_types": ["projects", "sections", "items"] });
        let value = self.post("/sync", &body)?;
        serde_json::from_value(value).map_err(|e| ApiError::Decode(e.to_string()))
    }

    /// Sends write commands. The endpoint takes them as a form field holding a
    /// JSON array, which is the shape the API documents for a write; the read
    /// above is the endpoint's other, separate, shape.
    pub fn sync_commands(&self, commands: &[serde_json::Value]) -> Result<SyncWrite, ApiError> {
        let encoded = serde_json::Value::Array(commands.to_vec()).to_string();
        let response = self
            .agent
            .post(format!("{}/sync", self.base))
            .header("Authorization", &format!("Bearer {}", self.token.expose()))
            .send_form([("commands", encoded.as_str())])
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        let value = read_json(response)?;
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
            .header("Authorization", &format!("Bearer {}", self.token.expose()))
            .send_json(body)
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        read_json(response)
    }
}

/// Only an unset variable means Todoist; a set one goes through the seam's check.
fn base_from(value: Result<String, std::env::VarError>) -> Result<String, String> {
    match value {
        Ok(value) => checked_base(&value),
        Err(std::env::VarError::NotPresent) => Ok(PRODUCTION_BASE.to_string()),
        Err(std::env::VarError::NotUnicode(_)) => Err(
            "DAM_TODOIST_BASE_URL is set to bytes that are not Unicode; the test seam takes \
             http://127.0.0.1:<port> or http://localhost:<port>"
                .to_string(),
        ),
    }
}

/// `DAM_TODOIST_BASE_URL` is a test seam. Only a loopback address is accepted,
/// so the variable cannot send the bearer token to another host.
fn checked_base(value: &str) -> Result<String, String> {
    let refused = || {
        format!(
            "DAM_TODOIST_BASE_URL is a test seam and must be http://127.0.0.1:<port> or \
             http://localhost:<port>, not {value:?}"
        )
    };
    let base = value.strip_suffix('/').unwrap_or(value);
    let (host, port) = base
        .strip_prefix("http://")
        .and_then(|rest| rest.rsplit_once(':'))
        .ok_or_else(refused)?;
    if !matches!(host, "127.0.0.1" | "localhost")
        || port.is_empty()
        || !port.chars().all(|c| c.is_ascii_digit())
    {
        return Err(refused());
    }
    Ok(base.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_url_seam_accepts_a_loopback_address_and_nothing_else() {
        assert_eq!(
            checked_base("http://127.0.0.1:8080").unwrap(),
            "http://127.0.0.1:8080"
        );
        assert_eq!(
            checked_base("http://localhost:8080/").unwrap(),
            "http://localhost:8080"
        );
        for refused in [
            "https://api.todoist.com/api/v1",
            "http://evil.test:8080",
            "http://127.0.0.1.evil.test:8080",
            "http://user@127.0.0.1:8080",
            "http://localhost:8080@evil.test",
            "http://127.0.0.1",
            "http://127.0.0.1:",
            "https://127.0.0.1:8080",
            "",
        ] {
            assert!(checked_base(refused).is_err(), "{refused}");
        }
    }

    /// A set variable is never read as unset, which would send the bearer
    /// token to Todoist while the operator meant a test server.
    #[test]
    fn a_base_url_that_is_not_unicode_is_refused() {
        let not_unicode = std::env::VarError::NotUnicode(std::ffi::OsString::from("x"));
        let err = base_from(Err(not_unicode)).unwrap_err();
        assert!(err.contains("DAM_TODOIST_BASE_URL"), "{err}");
        assert_eq!(
            base_from(Err(std::env::VarError::NotPresent)).unwrap(),
            PRODUCTION_BASE
        );
        assert_eq!(
            base_from(Ok("http://127.0.0.1:9/".into())).unwrap(),
            "http://127.0.0.1:9"
        );
    }

    #[test]
    fn the_api_never_debug_prints_its_token() {
        let api = TodoistApi::new("http://example.test", "SUPERSECRETTOKEN");
        assert!(!format!("{api:?}").contains("SUPERSECRETTOKEN"));
    }

    #[test]
    fn new_trims_a_trailing_slash_from_the_base() {
        let api = TodoistApi::new("http://example.test/", "tok");
        assert_eq!(api.base, "http://example.test");
    }
}
