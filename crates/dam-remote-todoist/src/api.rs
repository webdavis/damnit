use std::fmt;
use std::time::Duration;

use serde::Deserialize;

pub const PRODUCTION_BASE: &str = "https://api.todoist.com/api/v1";

/// How much of an upstream error body reaches dam, the ellipsis included. The
/// body crosses the protocol, is stored as a notice and is printed by
/// `dam status`, so it is cut to one line of at most this many characters.
pub const MAX_ERROR_BODY: usize = 500;

#[derive(Debug)]
pub enum ApiError {
    Http {
        status: u16,
        body: String,
    },
    /// Todoist is rate limiting this account. `retry_after` is its
    /// `Retry-After` header when it sent one.
    RateLimited {
        retry_after: Option<Duration>,
        body: String,
    },
    Transport(String),
    Decode(String),
}

impl ApiError {
    /// How long Todoist asked the caller to wait, when it said.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Http { status, body } => write!(f, "Todoist answered {status}: {body}"),
            ApiError::RateLimited {
                retry_after: Some(d),
                ..
            } => write!(
                f,
                "Todoist is rate limiting this account; retry after {}s",
                d.as_secs()
            ),
            ApiError::RateLimited {
                retry_after: None,
                body,
            } => write!(
                f,
                "Todoist is rate limiting this account and named no retry time: {body}"
            ),
            ApiError::Transport(s) => write!(f, "reaching Todoist: {s}"),
            ApiError::Decode(s) => write!(f, "reading Todoist's answer: {s}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// The bearer token. `Debug` redacts it and there is no `Display`, so `expose`
/// is the only way to the bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiToken(String);

impl ApiToken {
    /// The value itself. Call this only where it is sent as the Authorization header.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiToken(<redacted>)")
    }
}

impl From<String> for ApiToken {
    fn from(value: String) -> ApiToken {
        ApiToken(value)
    }
}

impl From<&str> for ApiToken {
    fn from(value: &str) -> ApiToken {
        ApiToken(value.to_string())
    }
}

#[derive(Debug)]
pub struct TodoistApi {
    agent: ureq::Agent,
    base: String,
    token: ApiToken,
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
        let base = match std::env::var("DAM_TODOIST_BASE_URL") {
            Ok(value) => checked_base(&value)?,
            Err(_) => PRODUCTION_BASE.to_string(),
        };
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
            .header("Authorization", &format!("Bearer {}", self.token.expose()))
            .send_json(body)
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        read_json(response)
    }

    pub fn delete(&self, path: &str) -> Result<(), ApiError> {
        let response = self
            .agent
            .delete(format!("{}{path}", self.base))
            .header("Authorization", &format!("Bearer {}", self.token.expose()))
            .call()
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        read_json(response).map(|_| ())
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

/// One line of at most `MAX_ERROR_BODY` characters. Every control character
/// becomes a space so a single notice cannot rewrite the terminal or spill
/// across lines, and a cut is marked with a trailing ellipsis.
fn bounded(text: &str) -> String {
    let flat = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>();
    let trimmed = flat.trim();
    match trimmed.char_indices().nth(MAX_ERROR_BODY - 1) {
        None => trimmed.to_string(),
        Some((cut, _)) => format!("{}\u{2026}", &trimmed[..cut]),
    }
}

/// Todoist's status for a rate limit.
const TOO_MANY_REQUESTS: u16 = 429;

fn read_json(response: ureq::http::Response<ureq::Body>) -> Result<serde_json::Value, ApiError> {
    let status = response.status().as_u16();
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse().ok())
        .map(Duration::from_secs);
    let text = response
        .into_body()
        .read_to_string()
        .map_err(|e| ApiError::Transport(e.to_string()))?;
    if status == TOO_MANY_REQUESTS {
        return Err(ApiError::RateLimited {
            retry_after,
            body: bounded(&text),
        });
    }
    if !(200..300).contains(&status) {
        return Err(ApiError::Http {
            status,
            body: bounded(&text),
        });
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
