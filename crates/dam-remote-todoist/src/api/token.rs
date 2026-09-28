use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct ApiToken(String);

impl ApiToken {
    pub fn bearer_authorization(&self) -> String {
        format!("Bearer {}", self.0)
    }
}

impl fmt::Debug for ApiToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiToken(<redacted>)")
    }
}

impl From<String> for ApiToken {
    fn from(value: String) -> ApiToken {
        ApiToken(value)
    }
}

impl From<&str> for ApiToken {
    fn from(value: &str) -> ApiToken {
        ApiToken(value.to_string())
    }
}
