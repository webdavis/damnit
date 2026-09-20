//! The bearer token, as a type that will not print itself.

use std::fmt;

/// The bearer token. `Debug` redacts it and there is no `Display`, so `expose`
/// is the only way to the bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiToken(String);

impl ApiToken {
    /// The value itself. Call this only where it is sent as the Authorization header.
    pub fn expose(&self) -> &str {
        &self.0
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
