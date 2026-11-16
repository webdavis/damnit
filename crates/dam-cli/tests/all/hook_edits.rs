use crate::support;

use support::errors::error_document;
use support::sandbox::Sandbox;

fn hooked(task_new: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_hooks(&[("task_new", task_new)]);
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

#[test]
fn an_edit_the_rules_refuse_with_refuse_changes_nothing_and_exits_4() {
    let _guard = support::guard("an_edit_the_rules_refuse_with_refuse_changes_nothing_and_exits_4");
    let sb = hooked("{ run = function() return { no_done = true } end, on_failure = 'refuse' }");
    let out = sb.output(&["new", "buy milk", "-o", "json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    let document = error_document(&out);
    assert_eq!(document["error"]["rule"], "hook_refused");
    let message = document["error"]["message"].as_str().unwrap();
    assert!(
        message.contains(": its edit request failed: ")
            && message.ends_with(
                "is not completed, so there is nothing to reopen; the command changed nothing"
            ),
        "{message}"
    );
    let (ok, out, err) = sb.dam(&["list", "-o", "json"]);
    assert!(ok, "{err}");
    let listing: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(listing["objects"], serde_json::json!([]));
}
