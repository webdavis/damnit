use crate::support;

use support::errors::{error_document, kind_of};
use support::sandbox::Sandbox;

#[test]
fn every_failure_class_names_itself_under_json() {
    let _guard = support::guard("every_failure_class_names_itself_under_json");
    let sb = Sandbox::new();
    assert_eq!(
        kind_of(&sb.output(&["show", "0000000", "-o", "json"])),
        "refused"
    );
    assert_eq!(kind_of(&sb.output(&["ls", "due:", "-o", "json"])), "parse");
    assert_eq!(kind_of(&sb.output(&["done", "zz", "-o", "json"])), "usage");

    std::fs::create_dir(sb.dir.path().join("wrong")).unwrap();
    let unopenable = sb
        .command(&["ls", "-o", "json"])
        .env("DAM_STORE", sb.dir.path().join("wrong"))
        .output()
        .unwrap();
    assert_eq!(kind_of(&unopenable), "store");

    let a_folder = sb.dir.path().join("folder.lua");
    std::fs::create_dir(&a_folder).unwrap();
    let unreadable_config = sb
        .command(&["ls", "-o", "json"])
        .env("DAM_CONFIG", &a_folder)
        .output()
        .unwrap();
    assert_eq!(kind_of(&unreadable_config), "store");
}

#[test]
fn toon_carries_the_same_error_document() {
    let _guard = support::guard("toon_carries_the_same_error_document");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);

    let as_json = error_document(&sb.output(&["done", &parent, "-o", "json"]))["error"].clone();
    let as_toon =
        String::from_utf8_lossy(&sb.output(&["done", &parent, "-o", "toon"]).stderr).into_owned();
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
        args.extend(["-o", "json"]);
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
