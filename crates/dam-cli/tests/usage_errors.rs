mod support;

use serde_json::{Value, json};
use support::errors::error_document;
use support::sandbox::Sandbox;
use support::schema::{assert_matches_schema, schema_errors};

const EXIT_USAGE: i32 = 2;

fn usage_document(sb: &Sandbox, args: &[&str]) -> Value {
    let out = sb.output(args);
    assert_eq!(out.status.code(), Some(EXIT_USAGE), "{args:?}");
    assert!(out.stdout.is_empty(), "{args:?} printed on stdout");
    let document = error_document(&out);
    assert_matches_schema(&document, "error.schema.json");
    document
}

fn usage_part(sb: &Sandbox, args: &[&str]) -> Value {
    let document = usage_document(sb, args);
    assert_eq!(document["error"]["kind"], "usage", "{document}");
    assert_eq!(document["error"]["rule"], Value::Null, "{document}");
    document["error"]["usage"].clone()
}

#[test]
fn each_problem_the_command_line_can_have_names_itself_and_its_word() {
    let _guard = support::guard("each_problem_the_command_line_can_have_names_itself_and_its_word");
    let sb = Sandbox::new();
    let lines: [(&[&str], &str, Value); 10] = [
        (
            &["agenda", "--bogus", "--json"],
            "unknown_flag",
            json!("--bogus"),
        ),
        (
            &["agenda", "--bogus=3", "--json"],
            "unknown_flag",
            json!("--bogus"),
        ),
        (
            &["nosuchcommand", "--json"],
            "unknown_subcommand",
            json!("nosuchcommand"),
        ),
        (
            &["new", "milk", "extra", "--json"],
            "unexpected_argument",
            json!("extra"),
        ),
        (
            &["new", "--json", "--", "milk", "-x"],
            "unexpected_argument",
            json!("-x"),
        ),
        (
            &["new", "milk", "--due", "--json"],
            "missing_value",
            json!("--due"),
        ),
        (
            &["new", "milk", "--priority", "9", "--json"],
            "bad_value",
            json!("--priority"),
        ),
        (
            &["token", "create", "tool", "--json"],
            "missing_argument",
            json!("--allow"),
        ),
        (
            &["edit", "1111", "--due", "today", "--no-due", "--json"],
            "conflict",
            json!("--due"),
        ),
        (&["agenda", "--no-pull=yes", "--json"], "other", Value::Null),
    ];
    for (line, problem, word) in lines {
        assert_eq!(
            usage_part(&sb, line),
            json!({"problem": problem, "word": word}),
            "{line:?}"
        );
    }
}

#[test]
fn a_missing_word_or_subcommand_has_no_word_to_name() {
    let _guard = support::guard("a_missing_word_or_subcommand_has_no_word_to_name");
    let sb = Sandbox::new();
    for line in [&["new", "--json"][..], &["--json"], &["token", "--json"]] {
        assert_eq!(
            usage_part(&sb, line),
            json!({"problem": "missing_argument", "word": null}),
            "{line:?}"
        );
    }
}

#[test]
fn the_message_is_the_sentence_without_the_usage_block() {
    let _guard = support::guard("the_message_is_the_sentence_without_the_usage_block");
    let sb = Sandbox::new();
    let document = usage_document(&sb, &["agenda", "--bogus", "--json"]);
    assert_eq!(
        document["error"]["message"],
        "unexpected argument '--bogus' found"
    );
    assert_eq!(document["error"]["oids"], json!([]));
}

#[test]
fn a_usage_error_dam_raises_itself_says_other() {
    let _guard = support::guard("a_usage_error_dam_raises_itself_says_other");
    let sb = Sandbox::new();
    assert_eq!(
        usage_part(&sb, &["new", "milk", "--no-pull", "--json"]),
        json!({"problem": "other", "word": null})
    );
}

#[test]
fn toon_prints_the_same_document() {
    let _guard = support::guard("toon_prints_the_same_document");
    let sb = Sandbox::new();
    let out = sb.output(&["agenda", "--bogus", "--toon"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE));
    let document: Value =
        toon_format::decode_default(&String::from_utf8_lossy(&out.stderr)).unwrap();
    assert_eq!(
        document,
        usage_document(&sb, &["agenda", "--bogus", "--json"])
    );
}

#[test]
fn text_mode_prints_what_clap_prints() {
    let _guard = support::guard("text_mode_prints_what_clap_prints");
    let sb = Sandbox::new();
    let out = sb.output(&["agenda", "--bogus"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE));
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "error: unexpected argument '--bogus' found\n\n  \
         tip: to pass '--bogus' as a value, use '-- --bogus'\n\n\
         Usage: dam agenda [OPTIONS] [QUERY]\n\n\
         For more information, try '--help'.\n"
    );
}

#[test]
fn help_and_version_are_not_errors_under_json() {
    let _guard = support::guard("help_and_version_are_not_errors_under_json");
    let sb = Sandbox::new();
    for line in [&["agenda", "--help"][..], &["--version"]] {
        let plain = sb.output(line);
        let json = sb.output(&[line, &["--json"]].concat());
        assert!(json.status.success(), "{line:?}");
        assert_eq!(json.stdout, plain.stdout, "{line:?}");
        assert!(json.stderr.is_empty(), "{line:?}");
    }
}

#[test]
fn the_schema_holds_usage_exactly_when_the_kind_is_usage() {
    let without_usage = json!({"error": {
        "kind": "usage", "rule": null, "message": "m", "oids": []
    }});
    let usage_on_a_refusal = json!({"error": {
        "kind": "refused", "rule": "blocked", "message": "m", "oids": [],
        "usage": {"problem": "other", "word": null}
    }});
    let unknown_problem = json!({"error": {
        "kind": "usage", "rule": null, "message": "m", "oids": [],
        "usage": {"problem": "typo", "word": null}
    }});
    for document in [without_usage, usage_on_a_refusal, unknown_problem] {
        assert!(
            !schema_errors(&document, "error.schema.json").is_empty(),
            "{document}"
        );
    }
}
