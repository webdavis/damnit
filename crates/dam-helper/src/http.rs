use std::time::Duration;

use crate::service_error::{ServiceError, one_line};

const CALL_DEADLINE: Duration = Duration::from_secs(30);

const REDIRECTS_FOLLOWED_SO_NO_CREDENTIAL_REACHES_ANOTHER_HOST: u32 = 0;

const LONGEST_ANSWER_ONE_PROTOCOL_LINE_CAN_DELIVER: u64 = dam_protocol::MAX_LINE;

const TOO_MANY_REQUESTS: u16 = 429;

pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(CALL_DEADLINE))
        .timeout_await_100(Some(Duration::ZERO))
        .max_redirects(REDIRECTS_FOLLOWED_SO_NO_CREDENTIAL_REACHES_ANOTHER_HOST)
        .http_status_as_error(false)
        .build()
        .into()
}

#[derive(Debug)]
pub struct Answer {
    pub status: u16,
    pub retry_after: Option<Duration>,
    pub body: String,
}

pub fn read(
    service: &'static str,
    response: ureq::http::Response<ureq::Body>,
) -> Result<Answer, ServiceError> {
    read_within(
        service,
        response,
        LONGEST_ANSWER_ONE_PROTOCOL_LINE_CAN_DELIVER,
    )
}

pub fn read_answer(
    service: &'static str,
    response: ureq::http::Response<ureq::Body>,
) -> Result<String, ServiceError> {
    let answer = read(service, response)?;
    match answer.status {
        TOO_MANY_REQUESTS => Err(ServiceError::RateLimited {
            service,
            retry_after: answer.retry_after,
            body: one_line(&answer.body),
        }),
        200..=299 => Ok(answer.body),
        status => Err(ServiceError::Http {
            service,
            status,
            body: one_line(&answer.body),
        }),
    }
}

fn read_within(
    service: &'static str,
    response: ureq::http::Response<ureq::Body>,
    max: u64,
) -> Result<Answer, ServiceError> {
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
            ureq::Error::BodyExceedsLimit(limit) => ServiceError::TooLarge { service, limit },
            other => ServiceError::transport(service, other),
        })?;
    Ok(Answer {
        status,
        retry_after,
        body,
    })
}

#[cfg(test)]
mod tests;
