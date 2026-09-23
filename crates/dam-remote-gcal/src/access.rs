//! The refresh_token grant: the three credentials in a form body, an access
//! token out. The body of a refused answer is where a token endpoint echoes
//! credentials back, so it is never quoted.

use crate::api_error::ApiError;
use crate::encoding::form;
use crate::http::{MAX_ANSWER, read};
use crate::oauth_error::oauth_error_code;
use crate::{Credentials, Endpoints, Secret};

pub fn access_token(
    agent: &ureq::Agent,
    endpoints: &Endpoints,
    credentials: &Credentials,
) -> Result<Secret, ApiError> {
    let body = form(&[
        ("grant_type", "refresh_token"),
        ("client_id", &credentials.client.id),
        ("client_secret", credentials.client.secret.expose()),
        ("refresh_token", credentials.refresh_token.expose()),
    ]);
    let refused = |status| ApiError::TokenRefused { status, code: None };
    let response = agent
        .post(&endpoints.token)
        .header("content-type", "application/x-www-form-urlencoded")
        .send(body.as_str())
        .map_err(|_| refused(None))?;
    let answer = read(response, MAX_ANSWER).map_err(|_| refused(None))?;
    if !(200..300).contains(&answer.status) {
        return Err(ApiError::TokenRefused {
            status: Some(answer.status),
            code: oauth_error_code(&answer.body),
        });
    }
    serde_json::from_str::<serde_json::Value>(&answer.body)
        .ok()
        .and_then(|v| {
            v.get("access_token")?
                .as_str()
                .filter(|t| !t.is_empty())
                .map(Secret::from)
        })
        .ok_or(refused(Some(answer.status)))
}
