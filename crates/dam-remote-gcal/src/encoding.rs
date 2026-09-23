//! Percent encoding both ways, form bodies, and unpadded base64url.

pub(crate) fn percent_encoded(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

pub(crate) fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (bytes[i], escaped) {
            (_, Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (b'+', None) => {
                out.push(b' ');
                i += 1;
            }
            (byte, None) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn form(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encoded(name), percent_encoded(value)))
        .collect::<Vec<_>>()
        .join("&")
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(crate) fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4648 section 10's vectors, in the URL-safe alphabet with no padding.
    #[test]
    fn base64url_matches_the_rfc_vectors() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg"),
            ("fo", "Zm8"),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg"),
            ("fooba", "Zm9vYmE"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64url(input.as_bytes()), expected, "{input}");
        }
        assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
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
        // `u8::from_str_radix` accepts `+f`; an escape still needs two hex digits.
        assert_eq!(percent_decoded("%+f"), "% f");
        assert_eq!(
            percent_decoded(&percent_encoded("4/0Ab_x&y=z")),
            "4/0Ab_x&y=z"
        );
    }

    /// A credential carrying `&` or `=` must not compose a field nobody wrote.
    #[test]
    fn a_form_body_encodes_names_and_values() {
        assert_eq!(form(&[("a", "1&b=2"), ("c d", "")]), "a=1%26b%3D2&c%20d=");
    }
}
