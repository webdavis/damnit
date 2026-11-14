use super::*;

#[test]
fn unpadded_base64url_matches_the_rfc_4648_section_10_vectors() {
    for (input, expected) in [
        ("", ""),
        ("f", "Zg"),
        ("fo", "Zm8"),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg"),
        ("fooba", "Zm9vYmE"),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(unpadded_base64url(input.as_bytes()), expected, "{input}");
    }
    assert_eq!(unpadded_base64url(&[0xfb, 0xff]), "-_8");
}

#[test]
fn percent_encoding_passes_the_unreserved_set_and_escapes_every_other_byte() {
    assert_eq!(percent_encoded("aZ09-._~"), "aZ09-._~");
    assert_eq!(
        percent_encoded("a b&c=d/é#@"),
        "a%20b%26c%3Dd%2F%C3%A9%23%40"
    );
}

#[test]
fn percent_decoding_reads_escapes_and_plus_and_keeps_a_broken_escape() {
    assert_eq!(percent_decoded("a%20b+c%2F"), "a b c/");
    assert_eq!(percent_decoded("100%zz%4"), "100%zz%4");
    assert_eq!(
        percent_decoded(&percent_encoded("4/0Ab_x&y=z")),
        "4/0Ab_x&y=z"
    );
}

#[test]
fn an_escape_needs_two_hex_digits_though_from_str_radix_takes_a_leading_plus() {
    assert_eq!(u8::from_str_radix("+f", 16), Ok(15));
    assert_eq!(percent_decoded("%+f"), "% f");
}

#[test]
fn a_form_body_encodes_names_and_values_so_an_ampersand_or_equals_composes_no_field() {
    assert_eq!(form(&[("a", "1&b=2"), ("c d", "")]), "a=1%26b%3D2&c%20d=");
}
