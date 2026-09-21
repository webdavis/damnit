//! The contract the clients read: the error document, one exit code per
//! failure class, and the shape of a change document.

mod support;

use support::sandbox::Sandbox;
use support::status_json;

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
    // The advisory is in the answer rather than lost: the operator still
    // learns their credential-bearing config had been readable by someone else.
    let answer: serde_json::Value =
        serde_json::from_slice(&added.stdout).expect("the answer is one document");
    let warnings = answer["warnings"].as_array().expect("warnings is a list");
    assert_eq!(warnings.len(), 1, "{answer}");
    let said = warnings[0].as_str().unwrap();
    assert!(said.contains("644"), "{said}");
    assert!(!said.contains('/'), "the advisory carries a path: {said}");

    let quiet = sb.output(&["remote", "add", "third", "fake::", "--json"]);
    let answer: serde_json::Value = serde_json::from_slice(&quiet.stdout).unwrap();
    assert_eq!(
        answer["warnings"],
        serde_json::json!([]),
        "an already-private config warns about nothing"
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

    let unset = sb.dir.path().join("no-credential.toml");
    std::fs::write(
        &unset,
        "[remote.fake]\nurl = \"fake::\"\ncredentials = [\"api_token\"]\napi_token_env = \"DAM_TEST_UNSET_TOKEN\"\n",
    )
    .unwrap();
    let no_credential = sb
        .command(&["push", "--json"])
        .env("DAM_CONFIG", &unset)
        .env_remove("DAM_TEST_UNSET_TOKEN")
        .output()
        .unwrap();
    assert_eq!(kind_of(&no_credential), "credential");
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
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);

    let as_json = error_document(&sb.output(&["done", &parent, "--json"]))["error"].clone();
    let as_toon =
        String::from_utf8_lossy(&sb.output(&["done", &parent, "--toon"]).stderr).into_owned();
    assert!(as_toon.contains("kind: refused"), "{as_toon}");
    assert!(as_toon.contains("rule: blocked"), "{as_toon}");
    assert!(as_toon.contains("oids[2]"), "{as_toon}");
    for oid in [&parent, &child] {
        assert!(as_toon.contains(oid), "{oid} is missing from {as_toon}");
    }
    let sentence = as_json["message"].as_str().unwrap().lines().next().unwrap();
    assert!(as_toon.contains(sentence), "{as_toon}");
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
    let mut distinct = named.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        named.len(),
        "two rules answered with one word through the binary"
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

/// A client names the commits a remote is owed and pairs them with the log, so
/// the row carries the commit ids in the order `dam log` lists them and the
/// count beside them is the length of that list.
#[test]
fn an_unpushed_row_names_its_commits_in_log_order() {
    let _guard = support::guard("an_unpushed_row_names_its_commits_in_log_order");
    let sb = Sandbox::new();
    for (subject, message) in [("first", "one"), ("second", "two")] {
        sb.new_object(&[subject]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", message]).0);
    }

    let (ok, out, err) = sb.dam(&["log", "--json"]);
    assert!(ok, "{err}");
    let log: serde_json::Value = serde_json::from_str(&out).unwrap();
    let logged: Vec<&str> = log["commits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(logged.len(), 2);

    let row = status_json(&sb, &["--json"])["unpushed"][0].clone();
    assert_eq!(row["remote"], "fake");
    assert_eq!(
        row["commit_ids"],
        serde_json::json!(logged),
        "the commit ids read in the order dam log lists them"
    );
    assert_eq!(
        row["commits"], 2,
        "the count is the length of the list beside it"
    );
}

/// The unpushed mark is drawn by matching a change row's object against this
/// list, so the row names the objects those commits touch as well as the
/// commits themselves, and an object two commits touched is named once.
#[test]
fn an_unpushed_row_names_each_object_once() {
    let _guard = support::guard("an_unpushed_row_names_each_object_once");
    let sb = Sandbox::new();
    let first = sb.new_object(&["first"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "one"]).0);
    let second = sb.new_object(&["second"]);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "two"]).0);
    assert!(sb.dam(&["edit", &second, "--priority", "1"]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "three"]).0);

    let row = status_json(&sb, &["--json"])["unpushed"][0].clone();
    assert_eq!(row["commits"], 3);
    let oids: Vec<String> = row["oids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        oids.len(),
        2,
        "the object two commits touched is named once: {oids:?}"
    );
    assert!(
        oids[0].starts_with(&second),
        "the object the newest commit touched comes first: {oids:?}"
    );
    assert!(
        oids[1].starts_with(&first),
        "then the one only an older commit touched: {oids:?}"
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
