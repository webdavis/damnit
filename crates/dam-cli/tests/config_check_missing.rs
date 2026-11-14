mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

fn checked(sb: &Sandbox) -> (String, serde_json::Value) {
    let (ok, text, err) = sb.dam(&["config", "check"]);
    assert!(ok, "{err}");
    let out = sb.output(&["config", "check", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_matches_schema(&answer, "config-check.schema.json");
    (text, answer)
}

fn set_mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn a_missing_secrets_lua_is_said_to_be_missing() {
    let _guard = support::guard("a_missing_secrets_lua_is_said_to_be_missing");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.secrets_path()).unwrap();
    let config = sb.config_path().display().to_string();
    let (text, answer) = checked(&sb);
    assert_eq!(text, format!("{config}\nno secrets.lua\n"));
    assert_eq!(answer["config"], config.as_str());
    assert_eq!(answer["secrets"], serde_json::Value::Null);
}

#[test]
fn a_config_lua_dam_could_not_create_is_said_to_be_missing() {
    let _guard = support::guard("a_config_lua_dam_could_not_create_is_said_to_be_missing");
    let sb = Sandbox::new();
    std::fs::remove_file(sb.config_path()).unwrap();
    std::fs::remove_file(sb.secrets_path()).unwrap();
    let folder = sb.config_path().parent().unwrap().to_path_buf();
    set_mode(&folder, 0o500);
    if std::fs::write(folder.join("probe"), "").is_ok() {
        set_mode(&folder, 0o700);
        eprintln!("skipped: this user writes into a folder marked read-only");
        return;
    }
    let (text, answer) = checked(&sb);
    set_mode(&folder, 0o700);
    assert_eq!(text, "no config.lua\nno secrets.lua\n");
    assert_eq!(answer["config"], serde_json::Value::Null);
    assert_eq!(answer["secrets"], serde_json::Value::Null);
}
