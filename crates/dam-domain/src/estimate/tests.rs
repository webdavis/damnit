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
