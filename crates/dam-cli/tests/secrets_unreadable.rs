mod support;

use std::os::unix::fs::PermissionsExt;

use support::errors::error_document;
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

#[test]
fn a_token_caller_is_refused_when_secrets_lua_cannot_be_read() {
    let _guard = support::guard("a_token_caller_is_refused_when_secrets_lua_cannot_be_read");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "task:create"])]);
    std::fs::set_permissions(sb.secrets_path(), std::fs::Permissions::from_mode(0o000)).unwrap();
    for line in [
        vec!["ls", "--json", "--no-pull"],
        vec!["new", "report", "--json"],
    ] {
        let out = sb.output_holding(&held, &line);
        assert_eq!(out.status.code(), Some(1), "{line:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{line:?}: {out:?}");
        assert_eq!(error_document(&out)["error"]["kind"], "store");
        assert!(!String::from_utf8_lossy(&out.stderr).contains(&held));
    }
}
