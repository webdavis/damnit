mod support;

use serde_json::json;
use support::request::{by_request, request_file, usage_refusal};
use support::sandbox::Sandbox;

fn nothing_stored(sb: &Sandbox) {
    let (ok, out, err) = sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(status["unstaged"], json!([]), "{status}");
}

#[test]
fn each_kind_of_bad_request_is_a_usage_error_naming_its_key() {
    let _guard = support::guard("each_kind_of_bad_request_is_a_usage_error_naming_its_key");
    let sb = Sandbox::new();
    let bad = [
        (
            &["new"][..],
            json!({"subject": "milk", "label": ["errand"]}),
            "label",
        ),
        (
            &["new"],
            json!({"subject": "milk", "labels": "errand"}),
            "labels",
        ),
        (&["new"], json!({"subject": 5}), "subject"),
        (&["new"], json!({"subject": "milk", "body": ["a"]}), "body"),
        (&["new"], json!({"due": "today"}), "subject"),
        (
            &["new"],
            json!({"subject": "milk", "priority": 5}),
            "priority",
        ),
        (
            &["new"],
            json!({"subject": "milk", "kind": "chore"}),
            "kind",
        ),
        (&["new"], json!({"subject": "milk", "json": true}), "json"),
        (
            &["new"],
            json!({"subject": "milk", "meta": {"session": 1}}),
            "meta",
        ),
        (&["edit"], json!({"oid": "1111", "no_due": "yes"}), "no_due"),
        (
            &["edit"],
            json!({"oid": "1111", "due": "today", "no_due": true}),
            "no_due",
        ),
        (&["edit"], json!({"oid": "1111", "editor": true}), "editor"),
        (&["done"], json!({"oid": "1111", "children": "up"}), "force"),
        (&["mv"], json!({"oid": "1111"}), "path"),
        (
            &["token", "create"],
            json!({"name": "vpt", "allow": []}),
            "allow",
        ),
        (
            &["token", "create"],
            json!({"name": "vpt", "allow": ["task:read"], "meta": {}}),
            "meta",
        ),
    ];
    for (command, request, key) in bad {
        let message = usage_refusal(&by_request(&sb, command, &request));
        assert!(message.contains(key), "{request}: {message}");
    }
    nothing_stored(&sb);
}

#[test]
fn a_request_that_is_not_an_object_is_a_usage_error() {
    let _guard = support::guard("a_request_that_is_not_an_object_is_a_usage_error");
    let sb = Sandbox::new();
    for document in ["[1]", "{not json", "\"milk\""] {
        let path = request_file(&sb, document);
        usage_refusal(&sb.output(&["new", "--json", "--request", &path]));
    }
    usage_refusal(&sb.output(&["new", "--json", "--request", "missing.json"]));
}

#[test]
fn a_field_in_the_request_and_on_the_command_line_is_a_usage_error_naming_it() {
    let _guard =
        support::guard("a_field_in_the_request_and_on_the_command_line_is_a_usage_error_naming_it");
    let sb = Sandbox::new();
    let path = request_file(
        &sb,
        r#"{"subject": "milk", "due": "today", "meta": {"a": "b"}}"#,
    );
    for flag in [&["--due", "today"][..], &["--meta", "a=c"]] {
        let line = [&["new", "--json", "--request", &path][..], flag].concat();
        let message = usage_refusal(&sb.output(&line));
        assert!(message.contains(&flag[0][2..]), "{message}");
    }
    nothing_stored(&sb);
}

#[test]
fn beside_a_request_only_global_flags_are_taken() {
    let _guard = support::guard("beside_a_request_only_global_flags_are_taken");
    let sb = Sandbox::new();
    let path = request_file(&sb, r#"{"subject": "milk"}"#);
    for beside in [&["--body", "oat"][..], &["-p", "1"], &["bread"]] {
        let line = [&["new", "--json", "--request", &path][..], beside].concat();
        usage_refusal(&sb.output(&line));
    }
    nothing_stored(&sb);
    let out = sb.output(&["--no-color", "new", "--json", "--request", &path]);
    assert!(out.status.success(), "{out:?}");
}

#[test]
fn a_request_for_a_command_without_a_request_schema_is_a_usage_error() {
    let _guard =
        support::guard("a_request_for_a_command_without_a_request_schema_is_a_usage_error");
    let sb = Sandbox::new();
    let path = request_file(&sb, "{}");
    let message = usage_refusal(&sb.output(&["ls", "--json", "--request", &path]));
    assert!(message.contains("--request"), "{message}");
}
