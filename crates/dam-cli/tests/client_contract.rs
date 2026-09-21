//! The contract the clients read: the error document, one exit code per
//! failure class, and the shape of a change document.

mod support;

use support::sandbox::Sandbox;

/// A machine format writes nothing to standard error but the one document a
/// failure produces, so this parses the whole stream rather than a line of it.
#[test]
fn a_machine_format_writes_nothing_to_stderr_but_the_document() {
    let _guard = support::guard("a_machine_format_writes_nothing_to_stderr_but_the_document");
    let sb = Sandbox::new();
    std::fs::set_permissions(
        sb.dir.path().join("config.toml"),
        std::os::unix::fs::PermissionsExt::from_mode(0o644),
    )
    .unwrap();

    let added = sb.output(&["remote", "add", "second", "fake::", "--json"]);
    assert_eq!(added.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&added.stderr),
        "",
        "a warning reached the stream a client parses as one document"
    );

    let failed = sb.output(&["remote", "add", "second", "fake::", "--json"]);
    let err = String::from_utf8_lossy(&failed.stderr);
    serde_json::from_str::<serde_json::Value>(&err)
        .unwrap_or_else(|e| panic!("stderr is not one JSON document: {e}\n{err}"));
}

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
    assert_eq!(document["error"]["rule"], "blocked");
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

/// `-e` is refused rather than run under a machine format, and an editor that
/// dies in a human run is an editor failure rather than a bad command line.
#[test]
fn an_editor_is_refused_under_a_machine_format_and_its_death_is_its_own_failure() {
    let _guard = support::guard(
        "an_editor_is_refused_under_a_machine_format_and_its_death_is_its_own_failure",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk"]);
    let dies = sb.dir.path().join("bin/dies");
    std::fs::write(&dies, "#!/bin/sh\nexit 1\n").unwrap();
    std::fs::set_permissions(&dies, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();

    let refused = sb
        .command(&["edit", &oid, "-e", "--json"])
        .env("EDITOR", &dies)
        .env("VISUAL", &dies)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(4));
    let document = error_document(&refused);
    assert_eq!(document["error"]["kind"], "refused");
    assert_eq!(document["error"]["rule"], "needs_an_editor");

    let died = sb
        .command(&["edit", &oid, "-e"])
        .env("EDITOR", &dies)
        .env("VISUAL", &dies)
        .output()
        .unwrap();
    assert_eq!(
        died.status.code(),
        Some(1),
        "an editor that died is dam failing"
    );
    let err = String::from_utf8_lossy(&died.stderr);
    assert!(err.starts_with("dam: editor: "), "{err}");
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
        (
            "a question no machine format can answer",
            vec!["done", &parent, "--force", "--interactive"],
        ),
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
    let mut named = Vec::new();
    for (rule, mut args) in refusals {
        args.push("--json");
        let out = sb.output(&args);
        assert_eq!(out.status.code(), Some(4), "{rule}: {args:?}");
        assert_eq!(kind_of(&out), "refused", "{rule}");
        named.push(
            error_document(&out)["error"]["rule"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        );
    }
    assert_eq!(
        named,
        vec![
            "nothing_to_commit",
            "needs_an_answer",
            "blocked",
            "move_inside_itself",
            "not_an_event"
        ],
        "each rule names itself"
    );
}

/// A client reads the field names off dam's answer instead of diffing the
/// before and after itself.
#[test]
fn a_change_document_names_the_fields_it_touches() {
    let _guard = support::guard("a_change_document_names_the_fields_it_touches");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["buy oat milk", "--due", "2026-09-25", "-p", "1"]);
    assert_eq!(
        unstaged_fields(&sb),
        serde_json::json!(["subject", "priority", "due"]),
        "a create names every field it sets"
    );

    commit_everything(&sb, "the create");
    assert!(sb.dam(&["edit", &oid, "--subject", "buy soy milk"]).0);
    assert_eq!(unstaged_fields(&sb), serde_json::json!(["subject"]));

    commit_everything(&sb, "the edit");
    assert!(sb.dam(&["done", &oid]).0);
    assert_eq!(unstaged_fields(&sb), serde_json::json!(["done"]));

    commit_everything(&sb, "the completion");
    assert!(sb.dam(&["edit", &oid, "--undone"]).0);
    assert_eq!(
        unstaged_fields(&sb),
        serde_json::json!(["done"]),
        "reopening moves done and nothing else"
    );

    assert!(sb.dam(&["rm", &oid]).0);
    assert_eq!(
        unstaged_fields(&sb),
        serde_json::json!([]),
        "a delete removes the object whole and names no field"
    );
}

/// A background poll runs `status` per render, so its default answer carries
/// a row per change rather than two whole objects.
#[test]
fn status_embeds_no_object_until_asked_for_one() {
    let _guard = support::guard("status_embeds_no_object_until_asked_for_one");
    let sb = Sandbox::new();
    let oid = sb.new_object(&[
        "buy oat milk",
        "--due",
        "2026-09-25",
        "-p",
        "1",
        "--label",
        "errand",
    ]);

    let row = status_json(&sb, &["--json"])["unstaged"][0].clone();
    assert!(
        row.get("before").is_none() && row.get("after").is_none(),
        "{row}"
    );
    assert!(row["oid"].as_str().unwrap().starts_with(&oid));
    assert_eq!(row["op"], "create");
    assert_eq!(row["kind"], "task");
    assert_eq!(row["subject"], "buy oat milk");
    assert_eq!(row["due"], "2026-09-25");
    assert_eq!(row["priority"], 1);
    assert_eq!(row["done"], false);
    assert_eq!(row["labels"], serde_json::json!(["errand"]));

    let full = status_json(&sb, &["--json", "--full"])["unstaged"][0].clone();
    assert_eq!(full["before"], serde_json::Value::Null);
    assert_eq!(full["after"]["subject"], "buy oat milk");
    assert_eq!(full["after"]["body"], "");
    for key in ["oid", "op", "fields", "subject", "due"] {
        assert_eq!(full[key], row[key], "--full dropped {key}");
    }
}

fn commit_everything(sb: &Sandbox, message: &str) {
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", message]).0);
}

fn unstaged_fields(sb: &Sandbox) -> serde_json::Value {
    status_json(sb, &["--json"])["unstaged"][0]["fields"].clone()
}

fn status_json(sb: &Sandbox, extra: &[&str]) -> serde_json::Value {
    let mut args = vec!["status"];
    args.extend_from_slice(extra);
    let (ok, out, err) = sb.dam(&args);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}
