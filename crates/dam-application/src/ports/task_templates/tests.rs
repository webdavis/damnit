use super::TemplateName;

#[test]
fn a_plain_file_name_is_a_template_name() {
    assert_eq!(TemplateName::parse("bug").unwrap().as_str(), "bug");
    assert_eq!(
        TemplateName::parse("weekly-review").unwrap().file_name(),
        "weekly-review.lua"
    );
}

#[test]
fn a_name_that_could_leave_the_folder_is_refused() {
    for name in [
        "",
        "../../x",
        "a/b",
        "a\\b",
        "..",
        "x..y",
        "bug\u{1b}[31m",
        "bug\n",
        "/etc/passwd",
    ] {
        assert!(TemplateName::parse(name).is_err(), "{name:?}");
    }
}
