use super::names_a_label;

#[test]
fn a_plain_name_names_a_label() {
    for name in ["fix-login", "feature/x", "a:b", "@x", "v1.2", "é"] {
        assert!(names_a_label(name), "{name:?}");
    }
}

#[test]
fn an_empty_name_or_one_with_space_control_quote_or_operator_does_not() {
    for name in [
        "",
        "bad name",
        "tab\there",
        "evil\u{1b}[31m",
        "a|b",
        "a&b",
        "!a",
        "(a",
        "a)",
        "say\"hi",
    ] {
        assert!(!names_a_label(name), "{name:?}");
    }
}
