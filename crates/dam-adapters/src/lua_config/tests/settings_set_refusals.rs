use dam_application::{ConfigError, SettingKey, SettingsFile, Uneditable};

use super::support::config;

fn refused(before: &str, key: &str, text: &str) -> ConfigError {
    let folder = config(before);
    let key = SettingKey::parse(key).unwrap();
    let error = folder
        .file()
        .set(&key, &key.value(text).unwrap())
        .unwrap_err();
    assert_eq!(folder.text("config.lua"), before);
    error
}

#[test]
fn a_part_built_by_code_is_refused_with_the_line_to_add_by_hand() {
    let before =
        "local remotes = { work = { url = \"work::\" } }\nreturn {\n  remote = remotes,\n}\n";
    assert_eq!(
        refused(before, "remote.work.deadline", "2m"),
        ConfigError::SettingNotEditable {
            key: "remote.work.deadline".into(),
            why: Uneditable::TableBuiltInCode,
            by_hand: "remote.work.deadline = \"2m\"".into(),
        }
    );
}

#[test]
fn a_config_that_returns_code_is_refused() {
    let before = "local c = { owner = \"Ada\" }\nreturn c\n";
    assert!(matches!(
        refused(before, "owner", "Grace"),
        ConfigError::SettingNotEditable {
            why: Uneditable::ReturnsBuiltInCode,
            ..
        }
    ));
}

#[test]
fn a_read_only_config_lua_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let before = "return {\n  owner = \"Ada\",\n}\n";
    let folder = config(before);
    let path = folder.path("config.lua");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    let key = SettingKey::parse("owner").unwrap();
    let error = folder
        .file()
        .set(&key, &key.value("Grace").unwrap())
        .unwrap_err();
    assert!(matches!(error, ConfigError::SettingReadOnly { .. }));
    assert_eq!(folder.text("config.lua"), before);
}

#[test]
fn an_edit_that_would_change_another_setting_is_refused() {
    let before = "local calls = 0\nlocal function me()\n  calls = calls + 1\n  return \"Ada\"\nend\nreturn {\n  owner = me(),\n  done = { interactive = calls == 1 },\n}\n";
    assert_eq!(
        refused(before, "owner", "Grace"),
        ConfigError::SettingEditWouldBreak {
            key: "owner".into()
        }
    );
}
