mod support;

use std::os::unix::fs::PermissionsExt;

use support::sandbox::Sandbox;

fn loosened(sb: &Sandbox) {
    std::fs::set_permissions(sb.secrets_path(), std::fs::Permissions::from_mode(0o644)).unwrap();
}

fn mode_of(sb: &Sandbox) -> u32 {
    std::fs::metadata(sb.secrets_path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777
}

fn holding_a_token(sb: &Sandbox) {
    let (ok, _, err) = sb.dam(&["token", "create", "tool", "--allow", "task:read"]);
    assert!(ok, "{err}");
}

fn assert_warned_on_stderr_only(sb: &Sandbox, args: &[&str]) -> String {
    loosened(sb);
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(!out.contains("warning"), "{out}");
    assert!(err.starts_with("warning: ") && err.contains("644"), "{err}");
    assert_eq!(mode_of(sb), 0o600);
    out.trim_end().to_string()
}

fn assert_json_keeps_warnings_and_stderr_empty(sb: &Sandbox, args: &[&str]) {
    loosened(sb);
    let mut args = args.to_vec();
    args.push("--json");
    let (ok, out, err) = sb.dam(&args);
    assert!(ok, "{err}");
    assert!(err.is_empty(), "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(answer["warnings"].as_array().unwrap().len(), 1, "{answer}");
}

#[test]
fn token_create_prints_only_a_token_that_authenticates() {
    let _guard = support::guard("token_create_prints_only_a_token_that_authenticates");
    let sb = Sandbox::new();
    let token =
        assert_warned_on_stderr_only(&sb, &["token", "create", "tool", "--allow", "task:read"]);
    let out = sb.output_holding(&token, &["ls", "--json", "--no-pull"]);
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn token_create_json_keeps_its_warnings() {
    let _guard = support::guard("token_create_json_keeps_its_warnings");
    let sb = Sandbox::new();
    assert_json_keeps_warnings_and_stderr_empty(
        &sb,
        &["token", "create", "tool", "--allow", "task:read"],
    );
}

#[test]
fn token_revoke_warns_on_stderr() {
    let _guard = support::guard("token_revoke_warns_on_stderr");
    let sb = Sandbox::new();
    holding_a_token(&sb);
    let out = assert_warned_on_stderr_only(&sb, &["token", "revoke", "tool"]);
    assert_eq!(out, "revoked token tool");
}

#[test]
fn token_revoke_json_keeps_its_warnings() {
    let _guard = support::guard("token_revoke_json_keeps_its_warnings");
    let sb = Sandbox::new();
    holding_a_token(&sb);
    assert_json_keeps_warnings_and_stderr_empty(&sb, &["token", "revoke", "tool"]);
}

#[test]
fn remote_add_keeps_the_mode_of_the_config_that_holds_no_secret() {
    let _guard = support::guard("remote_add_keeps_the_mode_of_the_config_that_holds_no_secret");
    let sb = Sandbox::new();
    std::fs::set_permissions(sb.config_path(), std::fs::Permissions::from_mode(0o644)).unwrap();
    let (ok, out, err) = sb.dam(&["remote", "add", "second", "fake::"]);
    assert!(ok, "{err}");
    assert_eq!(out, "added second (fake::)\n");
    assert!(err.is_empty(), "{err}");
    let mode = std::fs::metadata(sb.config_path())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o644);
}
