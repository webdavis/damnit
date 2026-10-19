mod support;

use std::os::unix::fs::PermissionsExt;

use support::sandbox::Sandbox;

const CAPABILITIES: &str = r#"{"protocol":1,"kinds":["task"],"fields":["subject"],"credentials":["api_token"],"incremental":false}"#;
const NOT_AN_OBJECT: &str =
    "dam: the helper answered something dam cannot read: expected a JSON object\n";

fn install_helper_answering(sb: &Sandbox, capabilities: &str, push: &str) {
    let script = format!(
        "#!/bin/sh\nwhile IFS= read -r line; do\n  case \"$line\" in\n    *'\"capabilities\"'*) printf '%s\\n' '{capabilities}' ;;\n    *'\"push\"'*) printf '%s\\n' '{push}' ;;\n    *) printf '%s\\n' '{{\"error\":\"unknown request\"}}' ;;\n  esac\ndone\n"
    );
    let helper = sb.dir.path().join("bin/dam-remote-fake");
    std::fs::write(&helper, script).unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn a_push_answered_with_an_array_is_refused_as_not_a_json_object() {
    let _guard = support::guard("a_push_answered_with_an_array_is_refused_as_not_a_json_object");
    let sb = Sandbox::new();
    install_helper_answering(&sb, CAPABILITIES, "[[]]");
    sb.new_object(&["buy oat milk"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "create"]).0);
    let out = sb.output(&["push"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stderr), NOT_AN_OBJECT);
}

#[test]
fn capabilities_answered_with_an_array_are_refused_as_not_a_json_object() {
    let _guard =
        support::guard("capabilities_answered_with_an_array_are_refused_as_not_a_json_object");
    let sb = Sandbox::new();
    install_helper_answering(&sb, r#"[1,["task"],["subject"],[],false]"#, "{}");
    let out = sb.output(&["pull"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stderr), NOT_AN_OBJECT);
}
