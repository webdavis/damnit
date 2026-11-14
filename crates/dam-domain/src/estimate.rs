pub fn parse_minutes(text: &str) -> Option<u32> {
    let (number, minutes_per_unit) = match text.strip_suffix('h') {
        Some(hours) => (hours, 60),
        None => (text.strip_suffix('m').unwrap_or(text), 1),
    };
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    number.parse::<u32>().ok()?.checked_mul(minutes_per_unit)
}

#[cfg(test)]
mod tests;
