//! The one word of a refused OAuth answer that may be repeated: RFC 6749
//! section 5.2's error codes. The rest of the body is where a refused
//! exchange echoes the credentials back, so it is never quoted.

const CODES: [&str; 6] = [
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
    CODES.into_iter().find(|code| *code == stated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_word_is_named_and_the_rest_of_the_body_is_not() {
        let body = r#"{"error":"invalid_grant","error_description":"Bad Request 1//0gSECRET"}"#;
        assert_eq!(oauth_error_code(body), Some("invalid_grant"));
        assert_eq!(oauth_error_code(r#"{"error":"1//0gSECRET"}"#), None);
        assert_eq!(oauth_error_code("not json"), None);
    }
}
