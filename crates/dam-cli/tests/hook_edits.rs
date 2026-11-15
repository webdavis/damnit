mod support;

use support::errors::error_document;
use support::sandbox::Sandbox;

fn hooked(task_new: &str) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_config_lua(&format!(
        "return {{ hook = {{\n  task_new = {task_new},\n}} }}\n"
    ));
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

fn only_task(sb: &Sandbox) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    let listing: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(listing["objects"].as_array().unwrap().len(), 1, "{listing}");
    listing["objects"][0].clone()
}

#[test]
fn a_hook_that_returns_a_label_labels_the_new_task() {
    let _guard = support::guard("a_hook_that_returns_a_label_labels_the_new_task");
    let sb = hooked("function() return { labels = { 'errand' } } end");
    let out = sb.output(&["new", "buy milk", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
    assert_eq!(only_task(&sb)["labels"], serde_json::json!(["errand"]));
}

#[test]
fn a_wrong_request_warns_and_the_new_task_stands_as_given() {
    let _guard = support::guard("a_wrong_request_warns_and_the_new_task_stands_as_given");
    let sb = hooked("function() return { colour = 'red' } end");
    let out = sb.output(&["new", "buy milk", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let config = sb.config_path().display().to_string();
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!(
            "warning: hook task_new failed at {config}:2: its edit request failed: \
             --request: colour is not a field it takes; the change stands\n"
        )
    );
    assert_eq!(only_task(&sb)["subject"], "buy milk");
}

#[test]
fn an_edit_the_rules_refuse_with_refuse_changes_nothing_and_exits_4() {
    let _guard = support::guard("an_edit_the_rules_refuse_with_refuse_changes_nothing_and_exits_4");
    let sb = hooked("{ run = function() return { no_done = true } end, on_failure = 'refuse' }");
    let out = sb.output(&["new", "buy milk", "--json"]);
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
    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    let listing: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(listing["objects"], serde_json::json!([]));
}
