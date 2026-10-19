use std::fmt;
use std::time::Duration;

pub const MAX_ERROR_BODY_CHARS: usize = 500;

const CUT_MARK: char = '\u{2026}';

#[derive(Debug)]
pub enum ServiceError {
    Http {
        service: &'static str,
        status: u16,
        body: String,
    },
    RateLimited {
        service: &'static str,
        retry_after: Option<Duration>,
        body: String,
    },
    Transport {
        service: &'static str,
        why: String,
    },
    Decode {
        service: &'static str,
        why: String,
    },
    TooLarge {
        service: &'static str,
        limit: u64,
    },
}

impl ServiceError {
    pub fn transport(service: &'static str, why: impl fmt::Display) -> ServiceError {
        ServiceError::Transport {
            service,
            why: why.to_string(),
        }
    }

    pub fn decode(service: &'static str, why: impl fmt::Display) -> ServiceError {
        ServiceError::Decode {
            service,
            why: why.to_string(),
        }
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            ServiceError::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Http {
                service,
                status,
                body,
            } => write!(f, "{service} answered {status}: {body}"),
            ServiceError::RateLimited {
                service,
                retry_after: Some(d),
                ..
            } => write!(
                f,
                "{service} is rate limiting this account; retry after {}s",
                d.as_secs()
            ),
            ServiceError::RateLimited {
                service,
                retry_after: None,
                body,
            } => write!(
                f,
                "{service} is rate limiting this account and named no retry time: {body}"
            ),
            ServiceError::Transport { service, why } => write!(f, "reaching {service}: {why}"),
            ServiceError::Decode { service, why } => write!(f, "reading {service}'s answer: {why}"),
            ServiceError::TooLarge { service, limit } => write!(
                f,
                "{service}'s answer is past the {limit} byte ceiling one protocol line can carry"
            ),
        }
    }
}

impl std::error::Error for ServiceError {}

pub fn one_line(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let trimmed = flat.trim();
    match trimmed.char_indices().nth(MAX_ERROR_BODY_CHARS - 1) {
        None => trimmed.to_string(),
        Some((cut, _)) => format!("{}{CUT_MARK}", &trimmed[..cut]),
    }
}

#[cfg(test)]
mod tests;
