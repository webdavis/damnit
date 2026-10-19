use std::fmt;

use dam_helper::ServiceError;

pub(crate) const SERVICE: &str = "Google Calendar";

#[derive(Debug)]
pub enum ApiError {
    Service(ServiceError),
    TokenRefused {
        status: Option<u16>,
        code: Option<&'static str>,
    },
    TooManyPages {
        limit: usize,
    },
}

impl From<ServiceError> for ApiError {
    fn from(e: ServiceError) -> ApiError {
        ApiError::Service(e)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Service(e) => e.fmt(f),
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
            ApiError::TooManyPages { limit } => write!(
                f,
                "Google Calendar kept paging past {limit} pages, so the pull stopped"
            ),
        }
    }
}

impl std::error::Error for ApiError {}
