pub(crate) const FIRST_FRAME_AFTER_MS: u64 = 100;

pub(crate) fn leaves_a_line(age_ms: u64, drawn: bool) -> bool {
    drawn || age_ms >= FIRST_FRAME_AFTER_MS
}

#[cfg(test)]
mod tests;
