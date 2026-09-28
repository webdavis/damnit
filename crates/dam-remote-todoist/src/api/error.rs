use std::fmt;
use std::time::Duration;

pub const MAX_ERROR_BODY_CHARS: usize = 500;

const CUT_MARK: char = '\u{2026}';

const MAX_UPSTREAM_BODY_BYTES: u64 = dam_protocol::MAX_LINE;

#[derive(Debug)]
pub enum ApiError {
    Http {
        status: u16,
        body: String,
    },
    RateLimited {
        retry_after: Option<Duration>,
        body: String,
    },
    Transport(String),
    Decode(String),
    PastOneSyncCeiling {
        ceiling_bytes: u64,
    },
}

impl ApiError {
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
            ApiError::PastOneSyncCeiling { ceiling_bytes } => write!(
                f,
                "Todoist's answer is past the {ceiling_bytes} byte ceiling one full sync can carry"
            ),
        }
    }
}

impl std::error::Error for ApiError {}

pub(super) fn one_short_line(text: &str) -> String {
    let flat = control_characters_as_spaces(text);
    let trimmed = flat.trim();
    match trimmed.char_indices().nth(MAX_ERROR_BODY_CHARS - 1) {
        None => trimmed.to_string(),
        Some((cut, _)) => format!("{}{CUT_MARK}", &trimmed[..cut]),
    }
}

fn control_characters_as_spaces(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

const TOO_MANY_REQUESTS: u16 = 429;

pub(super) fn read_json(
    response: ureq::http::Response<ureq::Body>,
) -> Result<serde_json::Value, ApiError> {
    read_json_within(response, MAX_UPSTREAM_BODY_BYTES)
}

fn read_json_within(
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
            ureq::Error::BodyExceedsLimit(ceiling_bytes) => {
                ApiError::PastOneSyncCeiling { ceiling_bytes }
            }
            other => ApiError::Transport(other.to_string()),
        })?;
    if status == TOO_MANY_REQUESTS {
        return Err(ApiError::RateLimited {
            retry_after,
            body: one_short_line(&text),
        });
    }
    if !(200..300).contains(&status) {
        return Err(ApiError::Http {
            status,
            body: one_short_line(&text),
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

    #[test]
    fn the_upstream_body_ceiling_is_the_protocol_line_not_ureqs_narrower_default() {
        assert_eq!(MAX_UPSTREAM_BODY_BYTES, dam_protocol::MAX_LINE);
    }

    #[test]
    fn a_body_past_the_ceiling_is_its_own_refusal_naming_the_ceiling_not_a_transport_failure() {
        let err = read_json_within(answer(r#"{"items":[1,2,3]}"#), 8).unwrap_err();
        assert!(
            matches!(err, ApiError::PastOneSyncCeiling { ceiling_bytes: 8 }),
            "{err:?}"
        );
        assert!(err.to_string().contains('8'), "{err}");
    }

    #[test]
    fn every_control_character_becomes_a_space_so_a_notice_cannot_rewrite_the_terminal() {
        assert_eq!(one_short_line("red\u{1b}[31m\ttext\r\n"), "red [31m text");
    }

    #[test]
    fn a_body_inside_the_ceiling_decodes() {
        let value = read_json_within(answer(r#"{"ok":true}"#), MAX_UPSTREAM_BODY_BYTES).unwrap();
        assert_eq!(value["ok"], serde_json::json!(true));
    }
}
