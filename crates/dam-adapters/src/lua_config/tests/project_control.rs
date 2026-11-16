use dam_application::ConfigError;

use super::project_support::ProjectFolder;

fn refusal_of(text: &str) -> (ProjectFolder, ConfigError) {
    let folder = ProjectFolder::with(text);
    let refused = folder.read().unwrap_err();
    (folder, refused)
}

fn assert_refused_as_holding_a_control_character(text: &str) {
    let (folder, refused) = refusal_of(text);
    assert_eq!(
        refused,
        ConfigError::HoldsAControlCharacter {
            file: folder.file()
        },
        "{text:?}"
    );
}

#[test]
fn an_escape_sequence_in_a_key_name_is_refused() {
    assert_refused_as_holding_a_control_character(r#"return { ["\27[2J"] = 1, path = "a" }"#);
}

#[test]
fn an_escape_sequence_in_the_path_is_refused() {
    assert_refused_as_holding_a_control_character(r#"return { path = "a\27]0;x\7" }"#);
}

#[test]
fn an_escape_sequence_in_a_person_only_key_is_refused() {
    assert_refused_as_holding_a_control_character(r#"return { ["token\27[31m"] = 1 }"#);
}

#[test]
fn a_carriage_return_and_a_c1_character_are_refused() {
    assert_refused_as_holding_a_control_character(r#"return { path = "a\rb" }"#);
    assert_refused_as_holding_a_control_character("return { path = \"a\u{9b}31m\" }");
}

#[test]
fn a_path_that_is_not_a_path_is_refused_naming_only_the_file() {
    let (folder, refused) = refusal_of(r#"return { path = "a//b" }"#);
    assert_eq!(
        refused,
        ConfigError::NotAPath {
            file: folder.file()
        }
    );
}
