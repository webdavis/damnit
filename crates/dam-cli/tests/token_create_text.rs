mod support;

use std::os::unix::fs::PermissionsExt;

use support::sandbox::Sandbox;

fn listed_key_id(sb: &Sandbox) -> String {
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    answer["tokens"][0]["key_id"].as_str().unwrap().to_string()
}

fn with_secrets_mode(sb: &Sandbox, mode: u32) {
    std::fs::set_permissions(sb.secrets_path(), std::fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn token_create_names_the_token_and_key_id_on_stderr_and_prints_the_secret_alone() {
    let _guard = support::guard(
        "token_create_names_the_token_and_key_id_on_stderr_and_prints_the_secret_alone",
    );
    let sb = Sandbox::new();
    with_secrets_mode(&sb, 0o600);
    let (ok, out, err) = sb.dam(&["token", "create", "tool", "--allow", "task:read"]);
    assert!(ok, "{err}");
    let secret = out.strip_suffix('\n').unwrap();
    assert!(
        secret.starts_with("damk_v1_") && !secret.contains('\n'),
        "{out:?}"
    );
    let key_id = listed_key_id(&sb);
    assert_eq!(&secret[8..20], key_id);
    assert_eq!(
        err,
        format!("dam: created token tool, key id {key_id}\ndam: the secret is shown this once\n")
    );
}

#[test]
fn token_create_prints_a_mode_warning_before_naming_the_token() {
    let _guard = support::guard("token_create_prints_a_mode_warning_before_naming_the_token");
    let sb = Sandbox::new();
    with_secrets_mode(&sb, 0o644);
    let (ok, _, err) = sb.dam(&["token", "create", "tool", "--allow", "task:read"]);
    assert!(ok, "{err}");
    let lines: Vec<&str> = err.lines().collect();
    assert_eq!(lines.len(), 3, "{err}");
    assert!(lines[0].starts_with("warning: "), "{err}");
    assert!(
        lines[1].starts_with("dam: created token tool, key id "),
        "{err}"
    );
    assert_eq!(lines[2], "dam: the secret is shown this once");
}
