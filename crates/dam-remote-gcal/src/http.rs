//! One agent configuration and one bounded read, shared by every Google call.

use std::fmt;
use std::time::Duration;

/// The most of any one answer read. dam receives a pull as one protocol line,
/// so an answer past that line could never be delivered.
pub(crate) const MAX_ANSWER: u64 = dam_protocol::MAX_LINE;

/// Thirty seconds a call, redirects refused (a redirect is how a credential
/// reaches a host nobody meant), and every status read as an answer.
pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .into()
}

pub(crate) struct Answer {
    pub(crate) status: u16,
    pub(crate) body: String,
}

pub(crate) enum ReadError {
    Transport(String),
    TooLarge { limit: u64 },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Transport(s) => write!(f, "the connection failed: {s}"),
            ReadError::TooLarge { limit } => {
                write!(f, "the answer is past the {limit} byte ceiling")
            }
        }
    }
}

pub(crate) fn read(
    response: ureq::http::Response<ureq::Body>,
    max: u64,
) -> Result<Answer, ReadError> {
    let status = response.status().as_u16();
    let body = response
        .into_body()
        .into_with_config()
        .limit(max)
        .read_to_string()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(limit) => ReadError::TooLarge { limit },
            other => ReadError::Transport(other.to_string()),
        })?;
    Ok(Answer { status, body })
}
