//! RFC 7636: the verifier this run keeps, and the S256 challenge it shows.

use sha2::Digest;

use super::SignInError;
use crate::encoding::base64url;

pub(super) fn challenge(verifier: &str) -> String {
    base64url(sha2::Sha256::digest(verifier.as_bytes()).as_slice())
}

/// Thirty-two bytes from the operating system, as the forty-three characters
/// a verifier and a state are each written in.
pub(super) fn random_token() -> Result<String, SignInError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| SignInError::Random)?;
    Ok(base64url(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 7636 appendix B, the one published S256 example.
    #[test]
    fn the_challenge_is_the_rfc_example() {
        assert_eq!(
            challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    /// Thirty-two random bytes encode to forty-three characters, the shortest
    /// verifier RFC 7636 admits, and two draws differ.
    #[test]
    fn a_random_token_is_forty_three_url_safe_characters() {
        let a = random_token().unwrap();
        let b = random_token().unwrap();
        assert_eq!(a.len(), 43);
        assert!(
            a.bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        );
        assert_ne!(a, b);
    }
}
