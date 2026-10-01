use std::fmt;
use std::time::Duration;

pub(crate) const LONGEST_ANSWER_ONE_PROTOCOL_LINE_CAN_DELIVER: u64 = dam_protocol::MAX_LINE;

const CALL_DEADLINE: Duration = Duration::from_secs(30);

const REDIRECTS_FOLLOWED_SO_NO_CREDENTIAL_REACHES_ANOTHER_HOST: u32 = 0;

pub(crate) fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(CALL_DEADLINE))
        .max_redirects(REDIRECTS_FOLLOWED_SO_NO_CREDENTIAL_REACHES_ANOTHER_HOST)
        .http_status_as_error(false)
        .build()
        .into()
}

pub(crate) struct Answer {
    pub(crate) status: u16,
    pub(crate) retry_after: Option<Duration>,
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
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse().ok())
        .map(Duration::from_secs);
    let body = response
        .into_body()
        .into_with_config()
        .limit(max)
        .read_to_string()
        .map_err(|e| match e {
            ureq::Error::BodyExceedsLimit(limit) => ReadError::TooLarge { limit },
            other => ReadError::Transport(other.to_string()),
        })?;
    Ok(Answer {
        status,
        retry_after,
        body,
    })
}
