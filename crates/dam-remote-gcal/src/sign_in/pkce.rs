use sha2::Digest;

use super::SignInError;
use crate::encoding::unpadded_base64url;

const RANDOM_BYTES_ENCODING_TO_THE_SHORTEST_RFC_7636_VERIFIER: usize = 32;

pub(super) fn s256_challenge(verifier: &str) -> String {
    unpadded_base64url(sha2::Sha256::digest(verifier.as_bytes()).as_slice())
}

pub(super) fn random_token() -> Result<String, SignInError> {
    let mut bytes = [0u8; RANDOM_BYTES_ENCODING_TO_THE_SHORTEST_RFC_7636_VERIFIER];
    getrandom::fill(&mut bytes).map_err(|_| SignInError::Random)?;
    Ok(unpadded_base64url(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_challenge_is_the_rfc_7636_appendix_b_example() {
        assert_eq!(
            s256_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn a_random_token_is_the_shortest_rfc_7636_verifier_and_two_draws_differ() {
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
