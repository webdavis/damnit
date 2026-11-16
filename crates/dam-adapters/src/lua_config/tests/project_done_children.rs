use dam_application::{ChildDisposition, ConfigError, ProjectRuleFault};

use super::project_support::ProjectFolder;

#[test]
fn done_children_takes_the_words_of_children_on_the_command_line() {
    for (text, want) in [
        ("up", ChildDisposition::Up),
        ("keep", ChildDisposition::Keep),
        (
            "into: leftovers",
            ChildDisposition::Into("leftovers".into()),
        ),
    ] {
        let folder = ProjectFolder::with(&format!(
            "return {{ path = 'a/', done_children = '{text}' }}"
        ));
        assert_eq!(folder.read().unwrap().done_children, Some(want), "{text}");
    }
    let folder = ProjectFolder::with("return { path = 'a/' }");
    assert_eq!(folder.read().unwrap().done_children, None);
}

#[test]
fn a_wrong_done_children_is_refused_naming_only_the_file_and_the_key() {
    for text in [
        "return { path = 'a/', done_children = 'drop' }",
        "return { path = 'a/', done_children = 'into:' }",
        "return { path = 'a/', done_children = true }",
    ] {
        let folder = ProjectFolder::with(text);
        assert_eq!(
            folder.read(),
            Err(ConfigError::ProjectRule {
                file: folder.file(),
                key: "done_children",
                fault: ProjectRuleFault::Shape,
            }),
            "{text}"
        );
    }
}

#[test]
fn a_group_name_holding_a_control_character_is_refused() {
    let folder = ProjectFolder::with("return { path = 'a/', done_children = 'into:a\\27[2J' }");
    assert_eq!(
        folder.read(),
        Err(ConfigError::HoldsAControlCharacter {
            file: folder.file(),
        })
    );
}
