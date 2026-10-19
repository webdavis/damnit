use crate::api_error::{ApiError, SERVICE};
use crate::encoding::form;
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
    let answer = dam_helper::read(SERVICE, response).map_err(|_| refused(None))?;
    if !(200..300).contains(&answer.status) {
        return Err(ApiError::TokenRefused {
            status: Some(answer.status),
            code: oauth_error_code(&answer.body),
        });
    }
    let value: serde_json::Value = serde_json::from_str(&answer.body).map_err(|_| refused(None))?;
    value
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .filter(|t| !t.is_empty())
        .map(Secret::from)
        .ok_or(refused(Some(answer.status)))
}
