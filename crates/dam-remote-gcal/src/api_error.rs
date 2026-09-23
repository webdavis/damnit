//! What a call to Google can fail with, in words that never quote a
//! credential.

use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ApiError {
    /// The token endpoint refused, or answered without an access token.
    /// Only Google's RFC 6749 error word is kept, never the body.
    TokenRefused {
        status: Option<u16>,
        code: Option<&'static str>,
    },
    /// The body cut to one line of at most 500 characters.
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
    TooLarge {
        limit: u64,
    },
    TooManyPages {
        limit: usize,
    },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::TokenRefused {
                code: Some("invalid_grant"),
                status,
            } => write!(
                f,
                "Google refused the refresh token (HTTP {}, invalid_grant): it was revoked or has expired; run dam-gcal-sign-in again and store the new one (a consent screen still in \"Testing\" expires it after 7 days)",
                status.map_or_else(|| "?".to_string(), |s| s.to_string())
            ),
            ApiError::TokenRefused {
                status: Some(s),
                code: Some(c),
            } => write!(f, "Google refused the token exchange (HTTP {s}, {c})"),
            ApiError::TokenRefused {
                status: Some(s),
                code: None,
            } => write!(f, "Google refused the token exchange (HTTP {s})"),
            ApiError::TokenRefused { status: None, .. } => f.write_str(
                "the token exchange did not reach Google or its answer could not be read",
            ),
            ApiError::Http { status, body } => {
                write!(f, "Google Calendar answered {status}: {body}")
            }
            ApiError::RateLimited {
                retry_after: Some(d),
                ..
            } => write!(
                f,
                "Google Calendar is rate limiting this account; retry after {}s",
                d.as_secs()
            ),
            ApiError::RateLimited {
                retry_after: None,
                body,
            } => write!(
                f,
                "Google Calendar is rate limiting this account and named no retry time: {body}"
            ),
            ApiError::Transport(s) => write!(f, "reaching Google Calendar: {s}"),
            ApiError::Decode(s) => write!(f, "reading Google Calendar's answer: {s}"),
            ApiError::TooLarge { limit } => write!(
                f,
                "Google Calendar's answer is past the {limit} byte ceiling one pull can carry"
            ),
            ApiError::TooManyPages { limit } => write!(
                f,
                "Google Calendar kept paging past {limit} pages, so the pull stopped"
            ),
        }
    }
}
