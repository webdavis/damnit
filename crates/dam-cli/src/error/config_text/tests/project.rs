use dam_application::ConfigError;

use super::text;

#[test]
fn a_personal_key_in_a_project_file_is_sent_to_the_personal_config() {
    assert_eq!(
        text(ConfigError::BelongsInPersonalConfig {
            key: "token".into(),
            file: "/work/app/.dam.lua".into(),
        }),
        "config: token belongs in your own config.lua, never in a project file; \
         remove it from /work/app/.dam.lua"
    );
}

#[test]
fn a_project_file_refused_for_its_text_is_named_and_never_quoted() {
    let file = std::path::PathBuf::from("/work/app/.dam.lua");
    let held = text(ConfigError::HoldsAControlCharacter { file: file.clone() });
    assert_eq!(
        held,
        "config: /work/app/.dam.lua holds a control character in a key or a text; remove it"
    );
    let not_a_path = text(ConfigError::NotAPath { file });
    assert_eq!(
        not_a_path,
        "config: path in /work/app/.dam.lua is not a path such as work/app/; correct it"
    );
    for refusal in [held, not_a_path] {
        assert!(!refusal.chars().any(char::is_control), "{refusal:?}");
    }
}
