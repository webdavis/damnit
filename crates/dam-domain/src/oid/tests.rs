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
