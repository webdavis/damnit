mod support;

use dam_adapters::SqliteStore;
use dam_application::ObjectRepository;
use dam_domain::{Object, Oid, Task};
use serde_json::{Value, json};
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::{assert_matches_schema, schema_errors};

fn with_two_objects_starting_abab() -> Sandbox {
    let sb = Sandbox::new();
    let store = SqliteStore::open(&sb.dir.path().join("dam.db")).unwrap();
    for last in ["01", "02"] {
        let oid = Oid::parse(&format!("abab{}{last}", "0".repeat(34))).unwrap();
        store.put(&Object::Task(Task::new(oid, "milk"))).unwrap();
    }
    sb
}

fn error_of(sb: &Sandbox, args: &[&str], exit: i32) -> Value {
    let out = sb.output(args);
    assert_eq!(out.status.code(), Some(exit), "{args:?}");
    let document = error_document(&out);
    assert_matches_schema(&document, "error.schema.json");
    document["error"].clone()
}

#[test]
fn an_error_about_a_typed_id_names_the_text_as_given() {
    let _guard = support::guard("an_error_about_a_typed_id_names_the_text_as_given");
    let sb = with_two_objects_starting_abab();
    let cases: [(&[&str], i32, &str, Value, &str); 4] = [
        (
            &["done", "abab", "--json"],
            2,
            "usage",
            json!("ambiguous_prefix"),
            "abab",
        ),
        (
            &["show", "0000000", "--json"],
            4,
            "refused",
            json!("no_such_object"),
            "0000000",
        ),
        (
            &["done", "0000000", "--json"],
            4,
            "refused",
            json!("no_working_object"),
            "0000000",
        ),
        (&["done", "zz", "--json"], 2, "usage", Value::Null, "zz"),
    ];
    for (line, exit, kind, rule, prefix) in cases {
        let error = error_of(&sb, line, exit);
        assert_eq!(error["kind"], kind, "{error}");
        assert_eq!(error["rule"], rule, "{error}");
        assert_eq!(error["prefix"], prefix, "{error}");
        assert_eq!(error.get("usage"), None, "{error}");
    }
}

#[test]
fn a_whole_id_that_matches_nothing_is_named_by_its_first_seven_characters() {
    let _guard =
        support::guard("a_whole_id_that_matches_nothing_is_named_by_its_first_seven_characters");
    let sb = Sandbox::new();
    let whole = format!("1234567{}", "0".repeat(33));

    let error = error_of(&sb, &["done", &whole, "--json"], 4);

    assert_eq!(error["prefix"], "1234567", "{error}");
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("\"1234567\""), "{message}");
}

#[test]
fn an_ambiguous_prefix_still_lists_what_it_matched() {
    let _guard = support::guard("an_ambiguous_prefix_still_lists_what_it_matched");
    let sb = with_two_objects_starting_abab();
    let error = error_of(&sb, &["done", "abab", "--json"], 2);
    let matched: Vec<String> = ["01", "02"]
        .map(|last| format!("abab{}{last}", "0".repeat(34)))
        .to_vec();
    assert_eq!(error["oids"], json!(matched));
}

#[test]
fn a_refusal_about_no_typed_id_has_no_prefix() {
    let _guard = support::guard("a_refusal_about_no_typed_id_has_no_prefix");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    sb.new_object(&["child", "--path", "parent/"]);
    let error = error_of(&sb, &["done", &parent, "--json"], 4);
    assert_eq!(error["rule"], "blocked");
    assert_eq!(error.get("prefix"), None, "{error}");
}

#[test]
fn the_schema_holds_prefix_only_on_an_error_about_a_typed_id() {
    let on_a_blocked_refusal = json!({"error": {
        "kind": "refused", "rule": "blocked", "message": "m", "oids": [], "prefix": "abab"
    }});
    let on_a_store_failure = json!({"error": {
        "kind": "store", "rule": null, "message": "m", "oids": [], "prefix": "abab"
    }});
    let empty = json!({"error": {
        "kind": "refused", "rule": "no_such_object", "message": "m", "oids": [], "prefix": ""
    }});
    for document in [on_a_blocked_refusal, on_a_store_failure, empty] {
        assert!(
            !schema_errors(&document, "error.schema.json").is_empty(),
            "{document}"
        );
    }
}

#[test]
fn the_schema_holds_a_usage_error_to_one_of_usage_and_prefix() {
    let both = json!({"error": {
        "kind": "usage", "rule": "ambiguous_prefix", "message": "m", "oids": [],
        "prefix": "abab", "usage": {"problem": "other", "word": null}
    }});
    let neither = json!({"error": {
        "kind": "usage", "rule": null, "message": "m", "oids": []
    }});
    for document in [both, neither] {
        assert!(
            !schema_errors(&document, "error.schema.json").is_empty(),
            "{document}"
        );
    }
}
