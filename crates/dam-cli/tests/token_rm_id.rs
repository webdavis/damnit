mod support;

use support::request::by_request;
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

const EVERY_RIGHT: [&str; 3] = ["task:create", "task:read", "task:remove"];

fn tool(allow: &[&str]) -> (Sandbox, String) {
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(allow)]);
    (sb, held)
}

fn send(sb: &Sandbox, held: &str, session: &str) -> String {
    let meta = format!("session={session}");
    let line = [
        "new",
        "Fix the build",
        "--id",
        "build",
        "--meta",
        &meta,
        "--json",
    ];
    let sent = answer(&sb.output_holding(held, &line), "sent.schema.json");
    sent["oid"].as_str().unwrap().to_string()
}

fn take_back(sb: &Sandbox, held: &str, session: &str) -> serde_json::Value {
    let meta = format!("session={session}");
    let line = ["rm", "--id", "build", "--meta", &meta, "--json"];
    answer(&sb.output_holding(held, &line), "rm.schema.json")
}

fn owner(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

#[test]
fn a_waiting_item_is_withdrawn_by_its_id_and_a_repeat_hears_nothing() {
    let _guard = support::guard("a_waiting_item_is_withdrawn_by_its_id_and_a_repeat_hears_nothing");
    let (sb, held) = tool(&["task:create"]);
    let oid = send(&sb, &held, "a");
    let withdrawn = take_back(&sb, &held, "a");
    assert_eq!(withdrawn["answer"], "withdrawn");
    assert_eq!(withdrawn["item"], oid.as_str());
    assert_eq!(withdrawn["oid"], oid.as_str());
    let again = take_back(&sb, &held, "a");
    assert_eq!(
        again,
        serde_json::json!({"answer": "nothing", "item": null, "oid": null})
    );
}

#[test]
fn an_added_task_is_taken_back_through_one_removal_item() {
    let _guard = support::guard("an_added_task_is_taken_back_through_one_removal_item");
    let (sb, held) = tool(&EVERY_RIGHT);
    let oid = send(&sb, &held, "a");
    owner(&sb, &["add", &oid]);
    owner(&sb, &["commit", "-m", "build"]);
    let waiting = take_back(&sb, &held, "a");
    assert_eq!(waiting["answer"], "waiting");
    assert_eq!(waiting["oid"], oid.as_str());
    assert_eq!(take_back(&sb, &held, "a"), waiting);
    owner(&sb, &["add", waiting["item"].as_str().unwrap()]);
    assert_eq!(take_back(&sb, &held, "a")["answer"], "nothing");
}

#[test]
fn another_session_or_another_id_hears_nothing() {
    let _guard = support::guard("another_session_or_another_id_hears_nothing");
    let (sb, held) = tool(&EVERY_RIGHT);
    send(&sb, &held, "a");
    assert_eq!(take_back(&sb, &held, "b")["answer"], "nothing");
    let out = sb.output_holding(&held, &["rm", "--id", "other", "--meta", "session=a"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim_end(), "nothing");
    assert_eq!(take_back(&sb, &held, "a")["answer"], "withdrawn");
}

#[test]
fn removing_an_added_task_by_id_needs_task_remove() {
    let _guard = support::guard("removing_an_added_task_by_id_needs_task_remove");
    let (sb, held) = tool(&["task:create", "task:read"]);
    let oid = send(&sb, &held, "a");
    owner(&sb, &["add", &oid]);
    let line = ["rm", "--id", "build", "--meta", "session=a", "--json"];
    assert_eq!(refused(&sb.output_holding(&held, &line)), "not_permitted");
}

#[test]
fn the_owner_removes_his_own_task_by_id_and_hears_nothing_after() {
    let _guard = support::guard("the_owner_removes_his_own_task_by_id_and_hears_nothing_after");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["Fix the build", "--id", "build"]);
    let out = sb.output(&["rm", "--id", "build", "--json"]);
    let removed = answer(&out, "rm.schema.json");
    assert!(removed["oid"].as_str().unwrap().starts_with(&oid));
    let out = sb.output(&["rm", "--id", "build", "--json"]);
    assert_eq!(answer(&out, "rm.schema.json")["answer"], "nothing");
}

#[test]
fn an_oid_and_an_id_together_or_neither_is_a_usage_error() {
    let _guard = support::guard("an_oid_and_an_id_together_or_neither_is_a_usage_error");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["Fix the build"]);
    for (line, problem, word) in [
        (
            &["rm", &oid, "--id", "build", "--json"][..],
            "conflict",
            serde_json::json!("--id"),
        ),
        (
            &["rm", "--json"],
            "missing_argument",
            serde_json::Value::Null,
        ),
    ] {
        let out = sb.output(line);
        assert_eq!(out.status.code(), Some(2), "{line:?}");
        let usage = &support::errors::error_document(&out)["error"]["usage"];
        assert_eq!(
            usage,
            &serde_json::json!({"problem": problem, "word": word}),
            "{line:?}"
        );
    }
    let out = by_request(&sb, &["rm"], &serde_json::json!({"id": "build"}));
    assert_eq!(answer(&out, "rm.schema.json")["answer"], "nothing");
}

#[test]
fn an_id_whose_task_left_the_grants_answers_as_one_never_sent() {
    let _guard = support::guard("an_id_whose_task_left_the_grants_answers_as_one_never_sent");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&EVERY_RIGHT).paths(&["inbox"])]);
    let line = [
        "new",
        "Fix the build",
        "--path",
        "inbox",
        "--id",
        "build",
        "--json",
    ];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let oid = sent["oid"].as_str().unwrap();
    owner(&sb, &["add", oid]);
    owner(&sb, &["mv", oid, "home"]);
    let hidden = sb.output_holding(&held, &["rm", "--id", "build", "--json"]);
    let never = sb.output_holding(&held, &["rm", "--id", "other", "--json"]);
    assert_eq!(hidden, never);
    assert_eq!(answer(&hidden, "rm.schema.json")["answer"], "nothing");
}
