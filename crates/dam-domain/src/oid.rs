use std::fmt;

const BYTES: usize = 20;
const HEX_LEN: usize = BYTES * 2;
const SHORT_LEN: usize = 7;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Oid(String);

#[derive(Debug, PartialEq, Eq)]
pub enum OidError {
    Length(usize),
    NotHex,
}

impl Oid {
    pub fn generate(fill_with_entropy: &mut dyn FnMut(&mut [u8])) -> Oid {
        let mut bytes = [0u8; BYTES];
        fill_with_entropy(&mut bytes);
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
mod tests;
