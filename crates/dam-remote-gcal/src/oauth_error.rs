const RFC_6749_TOKEN_ERROR_CODES: [&str; 6] = [
    "invalid_request",
    "invalid_client",
    "invalid_grant",
    "unauthorized_client",
    "unsupported_grant_type",
    "invalid_scope",
];

pub(crate) fn oauth_error_code(body: &str) -> Option<&'static str> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let stated = value.get("error")?.as_str()?;
    RFC_6749_TOKEN_ERROR_CODES
        .into_iter()
        .find(|code| *code == stated)
}

#[cfg(test)]
mod tests;
