use crate::support;

use support::errors::error_document;
use support::sandbox::Sandbox;

#[test]
fn a_name_with_nothing_to_test_is_a_usage_error_naming_the_parts() {
    let _guard = support::guard("a_name_with_nothing_to_test_is_a_usage_error_naming_the_parts");
    let sb = Sandbox::new();
    for name in ["config", "database", "helpers", "pushes", "nope"] {
        let out = sb.output(&["test", name]);
        assert_eq!(out.status.code(), Some(2), "{name}: {out:?}");
        assert!(out.stdout.is_empty(), "{out:?}");
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(
            said.contains(name) && said.contains("credentials"),
            "{said}"
        );
    }
    let out = sb.output(&["test", "config", "-o", "json"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(error_document(&out)["error"]["kind"], "usage");
}

#[test]
fn a_token_holder_is_refused_before_anything_is_read() {
    let _guard = support::guard("a_token_holder_is_refused_before_anything_is_read");
    let sb = Sandbox::new();
    let calls = sb.dir.path().join("calls");
    sb.write_secrets_lua(&format!(
        "io.open({:?}, 'a'):write('x')\nreturn {{ schema = 1 }}\n",
        calls.display().to_string()
    ));
    let out = sb.output_holding("damk_v1_whatever", &["test", "-o", "json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    assert_eq!(error_document(&out)["error"]["rule"], "not_open_to_tokens");
    assert!(!calls.exists());
}
