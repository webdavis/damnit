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
mod tests;
