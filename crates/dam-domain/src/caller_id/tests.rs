use super::*;

#[test]
fn an_id_is_any_one_line_text_up_to_the_cap() {
    for good in [
        "dentist",
        "PR #12: review",
        "état",
        &"x".repeat(CALLER_ID_CAP),
    ] {
        assert_eq!(CallerId::parse(good).unwrap().as_str(), good);
    }
}

#[test]
fn an_empty_long_or_broken_id_is_refused() {
    assert_eq!(CallerId::parse(""), Err(CallerIdError::Empty));
    assert_eq!(
        CallerId::parse(&"é".repeat(CALLER_ID_CAP + 1)),
        Err(CallerIdError::TooLong)
    );
    for broken in ["a\nb", "a\rb", "a\u{2028}b", "a\tb"] {
        assert_eq!(CallerId::parse(broken), Err(CallerIdError::LineBreak));
    }
}
