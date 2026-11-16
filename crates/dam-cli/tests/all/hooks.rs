use crate::support;

use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const PRINTING: &[(&str, &str)] = &[
    (
        "task_new",
        "function(task) print('new ' .. task.subject) end",
    ),
    (
        "task_changed",
        "function(task) print('changed ' .. task.subject) end",
    ),
    (
        "task_done",
        "function(task) print('done ' .. task.subject) end",
    ),
];

fn hooked(hooks: &[(&str, &str)]) -> Sandbox {
    let sb = Sandbox::new();
    sb.write_hooks(hooks);
    sb.write_secrets_lua("return { schema = 1 }\n");
    sb
}

fn listed(sb: &Sandbox) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    let listing: serde_json::Value = serde_json::from_str(&out).unwrap();
    listing["objects"].clone()
}

#[test]
fn hooks_run_on_new_edit_and_done_and_print_to_standard_error() {
    let _guard = support::guard("hooks_run_on_new_edit_and_done_and_print_to_standard_error");
    let sb = hooked(PRINTING);
    let out = sb.output(&["new", "buy milk", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let created: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stderr), "new buy milk\n");
    let oid = created["oid"].as_str().unwrap();
    let (ok, _, err) = sb.dam(&["edit", oid, "--subject", "buy oat milk"]);
    assert!(ok, "{err}");
    assert_eq!(err, "changed buy oat milk\n");
    let (ok, _, err) = sb.dam(&["done", oid]);
    assert!(ok, "{err}");
    assert_eq!(err, "done buy oat milk\n");
}

#[test]
fn a_failing_hook_warns_and_the_change_stands() {
    let _guard = support::guard("a_failing_hook_warns_and_the_change_stands");
    let sb = hooked(&[("task_new", "function() error('no bell') end")]);
    let out = sb.output(&["new", "buy milk", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let hook = sb.hook_path("task_new").display().to_string();
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!("warning: hook task_new failed at {hook}:1: no bell; the change stands\n")
    );
    assert_eq!(listed(&sb).as_array().unwrap().len(), 1);
}

#[test]
fn a_failing_hook_that_refuses_changes_nothing_and_exits_4() {
    let _guard = support::guard("a_failing_hook_that_refuses_changes_nothing_and_exits_4");
    let sb = hooked(&[(
        "task_new",
        "{ run = function() return 1 end, on_failure = 'refuse' }",
    )]);
    let out = sb.output(&["new", "buy milk", "--json"]);
    assert_eq!(out.status.code(), Some(4), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    let document = error_document(&out);
    assert_matches_schema(&document, "error.schema.json");
    assert_eq!(document["error"]["rule"], "hook_refused");
    assert_eq!(document["error"]["hook"], "task_new");
    let hook = sb.hook_path("task_new").display().to_string();
    assert_eq!(
        document["error"]["message"],
        format!(
            "hook task_new failed at {hook}:1: it returned an integer, not nil or a table; \
             the command changed nothing"
        )
    );
    assert_eq!(listed(&sb), serde_json::json!([]));
}
