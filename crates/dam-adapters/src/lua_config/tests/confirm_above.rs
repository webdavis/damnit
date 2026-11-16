use dam_application::{Expected, Misread};

use super::support::{config, top_of};

#[test]
fn confirm_above_is_absent_unless_written() {
    assert_eq!(config("return {}").load().unwrap().confirm_above, None);
    assert_eq!(
        config("return { confirm_above = 10 }")
            .load()
            .unwrap()
            .confirm_above,
        Some(10)
    );
}

#[test]
fn confirm_above_refuses_anything_but_a_whole_number() {
    for written in ["'ten'", "-1", "2.5"] {
        assert_eq!(
            config(&format!("return {{ confirm_above = {written} }}")).refusal(),
            Misread::WrongType {
                key: "confirm_above".into(),
                at: top_of("config.lua"),
                expected: Expected::WholeNumber,
            }
            .into(),
            "{written}"
        );
    }
}
