mod support;

use support::errors::error_document;
use support::sandbox::Sandbox;

#[test]
fn a_remote_config_lua_holds_is_refused_as_a_rule_and_the_file_kept() {
    let _guard = support::guard("a_remote_config_lua_holds_is_refused_as_a_rule_and_the_file_kept");
    let sb = Sandbox::new();
    let before = std::fs::read_to_string(sb.config_path()).unwrap();
    let out = sb.output(&["remote", "add", "fake", "other::"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "dam: remote fake already exists\n"
    );
    let out = sb.output(&["remote", "add", "fake", "other::", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert_eq!(error_document(&out)["error"]["rule"], "remote_exists");
    assert_eq!(std::fs::read_to_string(sb.config_path()).unwrap(), before);
}
