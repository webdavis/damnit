use super::{Client, SignInError};
use crate::Secret;
use crate::token_grant::token_grant;

pub(super) fn exchange(
    agent: &ureq::Agent,
    token_endpoint: &str,
    client: &Client,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<Secret, SignInError> {
    token_grant(
        agent,
        token_endpoint,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &client.id),
            ("client_secret", client.secret.expose()),
            ("code_verifier", verifier),
            ("redirect_uri", redirect_uri),
        ],
        "refresh_token",
    )
    .map_err(|refusal| {
        if refusal.granted_without_the_field() {
            SignInError::NoRefreshToken
        } else {
            SignInError::Exchange(refusal)
        }
    })
}
