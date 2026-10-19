mod support;

use support::errors::{error_document, kind_of};
use support::sandbox::Sandbox;

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

    let message = document["error"]["message"].as_str().unwrap();
    let human = sb.output(&["done", &parent]);
    assert_eq!(
        String::from_utf8_lossy(&human.stderr).trim(),
        format!("dam: {message}").trim(),
        "the message is the sentence the human form prints, which keeps tokens and paths out"
    );
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

#[test]
fn every_refusal_exits_four_whatever_the_rule() {
    let _guard = support::guard("every_refusal_exits_four_whatever_the_rule");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    sb.new_object(&["child", "--path", "parent/"]);

    let refusals: Vec<(&str, Vec<&str>)> = vec![
        ("an empty stage", vec!["commit", "-m", "nothing"]),
        (
            "a question no machine format can answer",
            vec!["done", &parent, "--force", "--interactive"],
        ),
        ("a completion with an open child", vec!["done", &parent]),
        (
            "a move into its own path",
            vec!["mv", &parent, "parent/here"],
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
