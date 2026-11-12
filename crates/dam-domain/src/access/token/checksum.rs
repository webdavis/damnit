use super::base62;

pub(super) const LEN: usize = 6;
const IEEE_REVERSED_POLYNOMIAL: u32 = 0xEDB8_8320;

pub(super) fn of(text: &str) -> String {
    base62::padded(u64::from(crc32(text.as_bytes())), LEN)
}

pub(super) fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (IEEE_REVERSED_POLYNOMIAL & mask);
        }
    }
    !crc
}
