mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use support::sandbox::Sandbox;

fn set_mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn a_missing_secrets_lua_is_said_to_be_missing() {
    let _guard = support::guard("a_missing_secrets_lua_is_said_to_be_missing");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.secrets_path()).unwrap();
    let answer = sb.doctor(&["doctor", "config"]);
    assert_eq!(answer.code, Some(0));
    assert!(
        answer
            .out
            .contains("\n  ✓ config: config.lua is valid, no secrets.lua\n"),
        "{}",
        answer.out
    );
    let (_, document) = sb.doctor_json(&["doctor", "config"]);
    let config = &document["checks"][0];
    assert_eq!(config["config"], sb.config_path().display().to_string());
    assert_eq!(config["secrets"], serde_json::Value::Null);
}

#[test]
fn only_secrets_lua_is_said_to_be_read_beside_the_defaults() {
    let _guard = support::guard("only_secrets_lua_is_said_to_be_read_beside_the_defaults");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.config_path()).unwrap();
    sb.write_secrets_lua("return { schema = 1 }\n");
    let answer = sb.doctor(&["doctor", "config"]);
    assert!(
        answer
            .out
            .contains("\n  ✓ config: no config.lua, defaults in use; secrets.lua is valid\n"),
        "{}",
        answer.out
    );
}

#[test]
fn an_unreadable_config_lua_names_the_file_in_its_row() {
    let _guard = support::guard("an_unreadable_config_lua_names_the_file_in_its_row");
    let sb = Sandbox::new();
    set_mode(&sb.config_path(), 0o000);
    if std::fs::read(sb.config_path()).is_ok() {
        set_mode(&sb.config_path(), 0o600);
        eprintln!("skipped: this user reads a file marked unreadable");
        return;
    }
    let answer = sb.doctor(&["doctor", "config"]);
    set_mode(&sb.config_path(), 0o600);
    assert_eq!(answer.code, Some(1));
    let row = format!("\n  ✗ config: {}: ", sb.config_path().display());
    assert!(answer.out.contains(&row), "{}", answer.out);
    assert!(
        answer
            .out
            .contains("\n    → make the file it names readable\n"),
        "{}",
        answer.out
    );
}
