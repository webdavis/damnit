//! A credential or token, as a type that will not print itself.

use std::fmt;

/// A credential or token. `Debug` redacts it and there is no `Display`, so
/// `expose` is the only way to the bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// The value itself. Call this only where it is sent to Google or printed
    /// for the operator to keep.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Secret {
        Secret(value)
    }
}

impl From<&str> for Secret {
    fn from(value: &str) -> Secret {
        Secret(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_never_debug_prints_its_value() {
        let secret = Secret::from("1//0gSUPERSECRETREFRESH");
        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
        assert_eq!(secret.expose(), "1//0gSUPERSECRETREFRESH");
    }
}
