//! The authorization_code grant: the code and the verifier in, the refresh token out.

use super::{Client, SignInError};
use crate::Secret;
use crate::encoding::form;
use crate::http::{MAX_ANSWER, read};
use crate::oauth_error::oauth_error_code;

pub(super) fn exchange(
    agent: &ureq::Agent,
    token_endpoint: &str,
    client: &Client,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<Secret, SignInError> {
    let body = form(&[
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", &client.id),
        ("client_secret", client.secret.expose()),
        ("code_verifier", verifier),
        ("redirect_uri", redirect_uri),
    ]);
    let unreadable = || SignInError::Exchange {
        status: None,
        code: None,
    };
    let response = agent
        .post(token_endpoint)
        .header("content-type", "application/x-www-form-urlencoded")
        .send(body.as_str())
        .map_err(|_| unreadable())?;
    let answer = read(response, MAX_ANSWER).map_err(|_| unreadable())?;
    if !(200..300).contains(&answer.status) {
        return Err(SignInError::Exchange {
            status: Some(answer.status),
            code: oauth_error_code(&answer.body),
        });
    }
    let value: serde_json::Value = serde_json::from_str(&answer.body).map_err(|_| unreadable())?;
    value
        .get("refresh_token")
        .and_then(serde_json::Value::as_str)
        .filter(|t| !t.is_empty())
        .map(Secret::from)
        .ok_or(SignInError::NoRefreshToken)
}
