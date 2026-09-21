//! The contract the clients read: the error document, one exit code per
//! failure class, and the shape of a change document.

mod support;

use support::sandbox::Sandbox;

/// The error document a machine format prints, parsed off standard error.
fn error_document(out: &std::process::Output) -> serde_json::Value {
    let err = String::from_utf8_lossy(&out.stderr);
    serde_json::from_str(&err)
        .unwrap_or_else(|e| panic!("stderr is not one JSON document: {e}\n{err}"))
}

fn kind_of(out: &std::process::Output) -> String {
    error_document(out)["error"]["kind"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn a_refusal_under_json_is_a_document_naming_its_kind_message_and_oids() {
    let _guard =
        support::guard("a_refusal_under_json_is_a_document_naming_its_kind_message_and_oids");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);

    let out = sb.output(&["done", &parent, "--json"]);
    assert!(
        String::from_utf8_lossy(&out.stdout).is_empty(),
        "a failure wrote to standard output"
    );
    let document = error_document(&out);
    assert_eq!(document["error"]["kind"], "refused");
    let named: Vec<&str> = document["error"]["oids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o.as_str().unwrap())
        .collect();
    assert_eq!(named.len(), 2, "the task and then the blocker: {named:?}");
    assert!(
        named[0].starts_with(&parent) && named[0].len() == 40,
        "{named:?}"
    );
    assert!(
        named[1].starts_with(&child) && named[1].len() == 40,
        "{named:?}"
    );

    // The message is the sentence the human form prints, which is what carries
    // the rule that no token and no path ever reaches an error.
    let message = document["error"]["message"].as_str().unwrap();
    let human = sb.output(&["done", &parent]);
    assert_eq!(
        String::from_utf8_lossy(&human.stderr).trim(),
        format!("dam: {message}").trim()
    );
}

#[test]
fn a_human_run_keeps_its_plain_line_and_writes_no_document() {
    let _guard = support::guard("a_human_run_keeps_its_plain_line_and_writes_no_document");
    let sb = Sandbox::new();
    let err = String::from_utf8_lossy(&sb.output(&["show", "0000000"]).stderr).into_owned();
    assert!(err.starts_with("dam: "), "{err}");
    assert!(!err.contains('{'), "{err}");
}

#[test]
fn every_failure_class_names_itself_under_json() {
    let _guard = support::guard("every_failure_class_names_itself_under_json");
    let sb = Sandbox::new();
    assert_eq!(
        kind_of(&sb.output(&["show", "0000000", "--json"])),
        "refused"
    );
    assert_eq!(kind_of(&sb.output(&["ls", "due:", "--json"])), "parse");
    assert_eq!(kind_of(&sb.output(&["done", "zz", "--json"])), "usage");

    std::fs::create_dir(sb.dir.path().join("wrong")).unwrap();
    let unopenable = sb
        .command(&["ls", "--json"])
        .env("DAM_STORE", sb.dir.path().join("wrong"))
        .output()
        .unwrap();
    assert_eq!(kind_of(&unopenable), "store");

    let unreadable_config = sb
        .command(&["ls", "--json"])
        .env("DAM_CONFIG", sb.dir.path())
        .output()
        .unwrap();
    assert_eq!(kind_of(&unreadable_config), "store");

    let no_helper = sb
        .command(&["push", "--json"])
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert_eq!(kind_of(&no_helper), "helper");
}

#[test]
fn toon_carries_the_same_error_document() {
    let _guard = support::guard("toon_carries_the_same_error_document");
    let sb = Sandbox::new();
    let err =
        String::from_utf8_lossy(&sb.output(&["show", "0000000", "--toon"]).stderr).into_owned();
    assert!(err.contains("kind: refused"), "{err}");
}

/// Every rule dam keeps exits 4 and says `refused`, whichever rule it was.
/// A rule that reported some other code would make a client guess.
#[test]
fn every_refusal_exits_four_whatever_the_rule() {
    let _guard = support::guard("every_refusal_exits_four_whatever_the_rule");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);

    let refusals: Vec<(&str, Vec<&str>)> = vec![
        ("an empty stage", vec!["commit", "-m", "nothing"]),
        ("a completion with an open child", vec!["done", &parent]),
        (
            "a move into its own path",
            vec!["mv", &child, "parent/here"],
        ),
        (
            "an event field on a task",
            vec!["edit", &parent, "--start", "2026-09-25T10:00"],
        ),
    ];
    for (rule, mut args) in refusals {
        args.push("--json");
        let out = sb.output(&args);
        assert_eq!(out.status.code(), Some(4), "{rule}: {args:?}");
        assert_eq!(kind_of(&out), "refused", "{rule}");
    }
}
