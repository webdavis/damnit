//! What the Todoist API can refuse with, and how an upstream body is cut down
//! before it crosses into dam.

use std::fmt;
use std::time::Duration;

/// How much of an upstream error body reaches dam, the ellipsis included. The
/// body crosses the protocol, is stored as a notice and is printed by
/// `dam status`, so it is cut to one line of at most this many characters.
pub const MAX_ERROR_BODY: usize = 500;

/// The most bytes of any upstream answer this helper reads. dam receives a
/// pull as one protocol line, so a sync larger than that line could never be
/// delivered whatever was read of it. Naming it here keeps the helper from
/// inheriting ureq's own 10 MiB default, which is below the line and would
/// refuse a sync dam can still carry.
pub const MAX_UPSTREAM_BODY: u64 = dam_protocol::MAX_LINE;

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
    /// The account holds more than one full sync can carry. `limit` is the
    /// ceiling in bytes that the answer passed.
    TooLarge {
        limit: u64,
    },
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
            ApiError::TooLarge { limit } => write!(
                f,
                "Todoist's answer is past the {limit} byte ceiling one full sync can carry"
            ),
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
    read_json_bounded(response, MAX_UPSTREAM_BODY)
}

/// `read_json` with the ceiling named, so the threshold is tested without
/// building a body of the shipped size.
fn read_json_bounded(
    response: ureq::http::Response<ureq::Body>,
    max: u64,
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
        .into_with_config()
        .limit(max)
        .read_to_string()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(limit) => ApiError::TooLarge { limit },
            other => ApiError::Transport(other.to_string()),
        })?;
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

    fn answer(body: &str) -> ureq::http::Response<ureq::Body> {
        ureq::http::Response::builder()
            .status(200)
            .body(ureq::Body::builder().data(body))
            .unwrap()
    }

    /// dam's answer to a pull is one protocol line, so a sync body past that
    /// ceiling could never be delivered however much of it was read. Reading to
    /// exactly there keeps the helper's own limit from being the narrower one.
    #[test]
    fn the_upstream_body_ceiling_is_the_protocol_line() {
        assert_eq!(MAX_UPSTREAM_BODY, dam_protocol::MAX_LINE);
    }

    /// A body past the ceiling is its own refusal naming the limit, not the
    /// transport failure ureq's own default cap reports it as.
    #[test]
    fn a_body_past_the_ceiling_is_refused_by_name() {
        let err = read_json_bounded(answer(r#"{"items":[1,2,3]}"#), 8).unwrap_err();
        assert!(matches!(err, ApiError::TooLarge { limit: 8 }), "{err:?}");
        assert!(err.to_string().contains('8'), "{err}");
    }

    #[test]
    fn a_body_inside_the_ceiling_decodes() {
        let value = read_json_bounded(answer(r#"{"ok":true}"#), MAX_UPSTREAM_BODY).unwrap();
        assert_eq!(value["ok"], serde_json::json!(true));
    }
}
