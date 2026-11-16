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

#[test]
fn a_project_file_that_needs_a_newer_dam_names_both_versions() {
    assert_eq!(
        text(ConfigError::NeedsANewerDam {
            file: "/work/app/.dam.lua".into(),
            needs: "0.3.0".into(),
            this: "0.2.0".into(),
        }),
        "config: /work/app/.dam.lua needs dam 0.3.0 or newer; this is dam 0.2.0"
    );
}

#[test]
fn a_value_the_project_file_would_not_load_with_is_a_usage_error_said_once() {
    let refused = crate::error::CliError::Config(ConfigError::ProjectSettingWouldNotLoad {
        key: "path".into(),
        why: Box::new(ConfigError::NotAPath {
            file: "/work/app/.dam.lua".into(),
        }),
    });
    assert_eq!(
        refused.to_string(),
        "config: cannot set path: .dam.lua would not load with that value, so nothing was \
         changed; docs/lua.md lists what path takes"
    );
    assert_eq!(refused.exit_code(), 2);
}
