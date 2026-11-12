const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BYTES_PER_CHARACTER: usize = 8;

pub(super) fn all(text: &str) -> bool {
    text.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub(super) fn padded(mut n: u64, width: usize) -> String {
    let mut digits = vec![ALPHABET[0]; width];
    for slot in digits.iter_mut().rev() {
        *slot = ALPHABET[(n % 62) as usize];
        n /= 62;
    }
    String::from_utf8(digits).expect("the alphabet is ASCII")
}

pub(super) fn random_characters(
    count: usize,
    fill_with_entropy: &mut dyn FnMut(&mut [u8]),
) -> String {
    let mut bytes = vec![0u8; count * BYTES_PER_CHARACTER];
    fill_with_entropy(&mut bytes);
    bytes
        .as_chunks::<BYTES_PER_CHARACTER>()
        .0
        .iter()
        .map(|chunk| ALPHABET[(u64::from_le_bytes(*chunk) % 62) as usize] as char)
        .collect()
}
