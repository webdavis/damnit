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
