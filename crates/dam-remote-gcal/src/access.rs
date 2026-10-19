use crate::token_grant::token_grant;
use crate::{ApiError, Credentials, Endpoints, Secret};

pub fn access_token(
    agent: &ureq::Agent,
    endpoints: &Endpoints,
    credentials: &Credentials,
) -> Result<Secret, ApiError> {
    token_grant(
        agent,
        &endpoints.token,
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &credentials.client.id),
            ("client_secret", credentials.client.secret.expose()),
            ("refresh_token", credentials.refresh_token.expose()),
        ],
        "access_token",
    )
    .map_err(ApiError::TokenRefused)
}
