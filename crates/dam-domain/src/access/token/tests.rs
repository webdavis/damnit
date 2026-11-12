use super::*;

const VALID: &str = "damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf02emDFq";

fn with_checksum(body: &str) -> String {
    format!("{body}{}", checksum::of(body))
}

#[test]
fn the_checksum_is_the_ieee_crc32_of_the_text_in_six_base62_digits() {
    assert_eq!(checksum::crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(checksum::of(""), "000000");
    assert_eq!(checksum::of("123456789").len(), 6);
}

#[test]
fn a_well_formed_v1_token_parses_and_names_its_key_id() {
    let text = with_checksum("damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0");
    let token = Token::parse(&text).unwrap();
    assert_eq!(token.key_id().as_str(), "4fZq8LmT2xKa");
    assert_eq!(token.expose(), text);
}

#[test]
fn a_token_with_a_changed_character_fails_its_checksum() {
    let text = with_checksum("damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0");
    let tampered = text.replacen("Hc7", "Hc8", 1);
    assert_eq!(Token::parse(&tampered), Err(TokenError::Checksum));
}

#[test]
fn a_text_without_the_damk_prefix_is_not_a_token() {
    assert_eq!(Token::parse("ghp_abc"), Err(TokenError::NotAToken));
    assert_eq!(Token::parse(""), Err(TokenError::NotAToken));
}

#[test]
fn an_unknown_version_is_refused_by_name() {
    assert_eq!(
        Token::parse("damk_v2_4fZq8LmT2xKa_rest"),
        Err(TokenError::UnknownVersion("v2".into()))
    );
}

#[test]
fn a_version_that_is_not_a_short_v_and_digits_is_malformed_and_never_echoed() {
    for text in [
        "damk_s3cr3tValue_4fZq8LmT2xKa_rest",
        "damk_v_4fZq8LmT2xKa_rest",
        "damk_v12345_4fZq8LmT2xKa_rest",
        "damk_V2_4fZq8LmT2xKa_rest",
    ] {
        assert_eq!(Token::parse(text), Err(TokenError::Malformed), "{text}");
    }
}

#[test]
fn wrong_lengths_and_characters_outside_base62_are_malformed() {
    let short_key = with_checksum("damk_v1_4fZq8LmT2xK_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0");
    let long_secret = with_checksum("damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0X");
    let dash = with_checksum("damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8a-f0");
    let no_separator = "damk_v1_4fZq8LmT2xKaHc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0000000";
    for text in [short_key.as_str(), &long_secret, &dash, no_separator] {
        assert_eq!(Token::parse(text), Err(TokenError::Malformed), "{text}");
    }
    assert_eq!(Token::parse(VALID).err(), Some(TokenError::Checksum));
}

#[test]
fn a_generated_token_parses_back_to_itself() {
    let mut seed = 0u8;
    let token = Token::generate(&mut |buf: &mut [u8]| {
        for b in buf.iter_mut() {
            seed = seed.wrapping_mul(31).wrapping_add(7);
            *b = seed;
        }
    });
    assert_eq!(Token::parse(&token.expose()), Ok(token.clone()));
    assert!(token.expose().starts_with("damk_v1_"));
    assert_eq!(token.expose().len(), 59);
}

#[test]
fn a_constant_random_source_still_generates_a_token() {
    let token = Token::generate(&mut |buf: &mut [u8]| buf.fill(0xff));
    assert!(Token::parse(&token.expose()).is_ok());
}

#[test]
fn debug_names_the_key_id_and_never_the_secret() {
    let text = with_checksum("damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0");
    let shown = format!("{:?}", Token::parse(&text).unwrap());
    assert!(shown.contains("4fZq8LmT2xKa"), "{shown}");
    assert!(!shown.contains("Hc7pR2vN9wQe"), "{shown}");
}

#[test]
fn tokens_compare_equal_only_on_every_character() {
    let a = Token::parse(&with_checksum(
        "damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf0",
    ))
    .unwrap();
    let b = Token::parse(&with_checksum(
        "damk_v1_4fZq8LmT2xKa_Hc7pR2vN9wQe3sT6yBu1mXk4LdJ8aZf1",
    ))
    .unwrap();
    assert!(a.matches(&a.clone()));
    assert!(!a.matches(&b));
}
