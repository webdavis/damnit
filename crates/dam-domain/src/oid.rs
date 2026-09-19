use std::fmt;

const BYTES: usize = 20;
const HEX_LEN: usize = BYTES * 2;
const SHORT_LEN: usize = 7;

/// dam's own identifier for an object: 40 lowercase hex characters, assigned
/// at creation, shown as a 7 character prefix the way git shows an object name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Oid(String);

#[derive(Debug, PartialEq, Eq)]
pub enum OidError {
    Length(usize),
    NotHex,
}

impl Oid {
    /// The caller supplies entropy so this crate stays free of any source of it.
    pub fn generate(fill: &mut dyn FnMut(&mut [u8])) -> Oid {
        let mut bytes = [0u8; BYTES];
        fill(&mut bytes);
        Oid(bytes.iter().map(|b| format!("{b:02x}")).collect())
    }

    pub fn parse(text: &str) -> Result<Oid, OidError> {
        if text.len() != HEX_LEN {
            return Err(OidError::Length(text.len()));
        }
        if !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(OidError::NotHex);
        }
        Ok(Oid(text.to_ascii_lowercase()))
    }

    pub fn short(&self) -> &str {
        &self.0[..SHORT_LEN]
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for OidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OidError::Length(n) => write!(f, "an oid is 40 hex characters, got {n}"),
            OidError::NotHex => f.write_str("an oid holds only hex characters"),
        }
    }
}

impl std::error::Error for OidError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(byte: u8) -> impl FnMut(&mut [u8]) {
        move |buf| buf.fill(byte)
    }

    #[test]
    fn generate_is_forty_lowercase_hex_characters() {
        let oid = Oid::generate(&mut fixed(0xab));
        assert_eq!(oid.as_str().len(), 40);
        assert_eq!(oid.as_str(), "ab".repeat(20));
    }

    #[test]
    fn short_is_the_first_seven_characters() {
        let oid = Oid::generate(&mut fixed(0x12));
        assert_eq!(oid.short(), "1212121");
    }

    #[test]
    fn parse_accepts_forty_hex_and_normalizes_case() {
        let oid = Oid::parse(&"AB".repeat(20)).unwrap();
        assert_eq!(oid.as_str(), "ab".repeat(20));
    }

    #[test]
    fn parse_refuses_wrong_length() {
        assert_eq!(Oid::parse("abc"), Err(OidError::Length(3)));
    }

    #[test]
    fn parse_refuses_non_hex() {
        assert_eq!(Oid::parse(&"zz".repeat(20)), Err(OidError::NotHex));
    }

    #[test]
    fn display_is_the_full_string() {
        let oid = Oid::parse(&"0f".repeat(20)).unwrap();
        assert_eq!(oid.to_string(), "0f".repeat(20));
    }
}
