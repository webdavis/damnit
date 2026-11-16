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
