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
mod tests {
    use super::parse_minutes;

    #[test]
    fn minutes_read_as_a_bare_number_or_with_m_or_h() {
        assert_eq!(parse_minutes("5"), Some(5));
        assert_eq!(parse_minutes("5m"), Some(5));
        assert_eq!(parse_minutes("2h"), Some(120));
        for bad in ["", "m", "90s", "-5", "1.5h", "+5"] {
            assert_eq!(parse_minutes(bad), None, "{bad}");
        }
    }
}
