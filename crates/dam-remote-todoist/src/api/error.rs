//! What the Todoist API can refuse with, and how an upstream body is cut down
//! before it crosses into dam.

use std::fmt;
use std::time::Duration;

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

/// One line of at most `MAX_ERROR_BODY` characters. Every control character
/// becomes a space so a single notice cannot rewrite the terminal or spill
/// across lines, and a cut is marked with a trailing ellipsis.
pub(super) fn bounded(text: &str) -> String {
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

pub(super) fn read_json(
    response: ureq::http::Response<ureq::Body>,
) -> Result<serde_json::Value, ApiError> {
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
