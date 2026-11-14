mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn tool() -> (Sandbox, String) {
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "tool",
        1,
        vec![grant(&["task:read", "task:create", "task:change"])],
    );
    (sb, held)
}

fn send(sb: &Sandbox, held: &str, args: &[&str]) -> serde_json::Value {
    let mut line = vec!["new"];
    line.extend_from_slice(args);
    line.push("--json");
    answer(&sb.output_holding(held, &line), "sent.schema.json")
}

fn owner(sb: &Sandbox, args: &[&str]) -> String {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
    out
}

fn shown(sb: &Sandbox, oid: &str) -> serde_json::Value {
    serde_json::from_str(&owner(sb, &["show", oid, "--json", "--no-pull"])).unwrap()
}

#[test]
fn a_repeat_with_the_same_id_is_a_change_item_that_targets_the_waiting_task() {
    let _guard =
        support::guard("a_repeat_with_the_same_id_is_a_change_item_that_targets_the_waiting_task");
    let (sb, held) = tool();
    let first = send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    assert_eq!(first["answer"], "new");
    assert_eq!(first["id"], "dentist");
    let again = send(&sb, &held, &["Call the dentist", "--id", "dentist"]);
    assert_eq!(again["answer"], "changed");
    assert_eq!(again["item"], "change");
    assert_eq!(again["oid"], first["oid"]);
    assert_ne!(again["hash"], first["hash"]);
    assert_eq!(again["subject"], "Call the dentist");

    let resent = send(&sb, &held, &["Call the dentist", "--id", "dentist"]);
    assert_eq!(resent["answer"], "duplicate");
    assert_eq!(resent["hash"], again["hash"]);
}

#[test]
fn a_repeat_after_the_owner_took_the_task_leaves_the_owners_unsaved_edit_alone() {
    let _guard = support::guard(
        "a_repeat_after_the_owner_took_the_task_leaves_the_owners_unsaved_edit_alone",
    );
    let (sb, held) = tool();
    let first = send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    let oid = first["oid"].as_str().unwrap();
    owner(&sb, &["add", &oid[..7]]);
    owner(&sb, &["commit", "-m", "take it"]);
    let same = send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    assert_eq!(same["answer"], "duplicate");
    assert_eq!(same["oid"], oid);

    owner(&sb, &["edit", oid, "--body", "bring the forms"]);
    let resent = send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    assert_eq!(resent["answer"], "duplicate");
    let changed = send(&sb, &held, &["Call dentist", "--id", "dentist", "-p", "1"]);
    assert_eq!(changed["answer"], "changed");
    assert_eq!(changed["oid"], oid);
    let task = shown(&sb, oid);
    assert_eq!(task["body"], "bring the forms");
    assert_eq!(task["task"]["priority"], 4);

    owner(&sb, &["add", &changed["hash"].as_str().unwrap()[..7]]);
    let task = shown(&sb, oid);
    assert_eq!(task["body"], "bring the forms");
    assert_eq!(task["task"]["priority"], 1);
}

#[test]
fn two_tokens_with_the_same_id_make_two_tasks() {
    let _guard = support::guard("two_tokens_with_the_same_id_make_two_tasks");
    let (sb, held) = tool();
    let claude = grant_to(&sb, "claude", 2, vec![grant(&["task:create"])]);
    let from_tool = send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    let from_claude = send(&sb, &claude, &["Call dentist", "--id", "dentist"]);
    assert_eq!(from_claude["answer"], "new");
    assert_ne!(from_claude["oid"], from_tool["oid"]);
}

#[test]
fn the_same_subject_under_two_ids_makes_two_items() {
    let _guard = support::guard("the_same_subject_under_two_ids_makes_two_items");
    let (sb, held) = tool();
    let a = send(&sb, &held, &["Review the PR", "--id", "pr-1"]);
    let b = send(&sb, &held, &["Review the PR", "--id", "pr-2"]);
    assert_eq!(b["answer"], "new");
    assert_ne!(a["oid"], b["oid"]);
}

#[test]
fn a_declined_task_sent_again_under_its_id_is_refused_and_a_different_one_is_new() {
    let _guard = support::guard(
        "a_declined_task_sent_again_under_its_id_is_refused_and_a_different_one_is_new",
    );
    let (sb, held) = tool();
    let first = send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    owner(&sb, &["rm", first["hash"].as_str().unwrap()]);
    let line = ["new", "Call dentist", "--id", "dentist", "--json"];
    assert_eq!(refused(&sb.output_holding(&held, &line)), "item_declined");
    let other = send(&sb, &held, &["Call the dentist", "--id", "dentist"]);
    assert_eq!(other["answer"], "new");
    assert_ne!(other["oid"], first["oid"]);
}

#[test]
fn a_repeat_of_a_task_as_an_event_is_refused() {
    let _guard = support::guard("a_repeat_of_a_task_as_an_event_is_refused");
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "tool",
        1,
        vec![grant(&[
            "task:create",
            "task:change",
            "event:create",
            "event:change",
        ])],
    );
    send(&sb, &held, &["Call dentist", "--id", "dentist"]);
    let line = [
        "new",
        "Dentist",
        "--id",
        "dentist",
        "--event",
        "--start",
        "2026-09-25T14:00",
        "--end",
        "2026-09-25T15:00",
        "--json",
    ];
    assert_eq!(refused(&sb.output_holding(&held, &line)), "not_an_event");
}

#[test]
fn a_repeat_that_moves_the_task_is_refused_as_mv_is() {
    let _guard = support::guard("a_repeat_that_moves_the_task_is_refused_as_mv_is");
    let sb = Sandbox::new();
    let mover = grant_to(
        &sb,
        "claude",
        2,
        vec![grant(&["task:create", "task:change", "task:move"])],
    );
    send(&sb, &mover, &["Call dentist", "--id", "dentist"]);
    let line = [
        "new",
        "Call dentist",
        "--id",
        "dentist",
        "--path",
        "inbox",
        "--json",
    ];
    assert_eq!(
        refused(&sb.output_holding(&mover, &line)),
        "not_open_to_tokens"
    );
}
