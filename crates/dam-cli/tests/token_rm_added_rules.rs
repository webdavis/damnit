mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

const EVERY_RIGHT: [&str; 3] = ["task:create", "task:read", "task:remove"];

fn owner(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn added_from(sb: &Sandbox, held: &str, line: &[&str]) -> String {
    let mut line = line.to_vec();
    line.push("--json");
    let sent = answer(&sb.output_holding(held, &line), "sent.schema.json");
    let oid = sent["oid"].as_str().unwrap().to_string();
    owner(sb, &["add", &oid]);
    owner(sb, &["commit", "-m", "added"]);
    oid
}

fn removal(sb: &Sandbox, held: &str, oid: &str) -> serde_json::Value {
    answer(
        &sb.output_holding(held, &["rm", oid, "--json"]),
        "rm.schema.json",
    )
}

const MEETING: [&str; 7] = [
    "new",
    "Plan the week",
    "--event",
    "--start",
    "2026-09-25T14:00",
    "--end",
    "2026-09-25T15:00",
];

#[test]
fn removing_an_added_event_needs_event_remove() {
    let _guard = support::guard("removing_an_added_event_needs_event_remove");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["event:create", "event:read"])]);
    let oid = added_from(&sb, &held, &MEETING);
    let out = sb.output_holding(&held, &["rm", &oid, "--json"]);
    assert_eq!(refused(&out), "not_permitted");
    let rights = ["event:create", "event:read", "event:remove"];
    let held = grant_to(&sb, "tool", 1, vec![grant(&rights)]);
    assert_eq!(removal(&sb, &held, &oid)["answer"], "waiting");
}

#[test]
fn a_decline_holds_for_a_token_made_again_under_the_same_name() {
    let _guard = support::guard("a_decline_holds_for_a_token_made_again_under_the_same_name");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&EVERY_RIGHT)]);
    let oid = added_from(&sb, &held, &["new", "Call the dentist"]);
    let first = removal(&sb, &held, &oid);
    owner(&sb, &["rm", first["item"].as_str().unwrap()]);
    owner(&sb, &["token", "revoke", "tool"]);
    let again = grant_to(&sb, "tool", 2, vec![grant(&EVERY_RIGHT)]);
    let answer = removal(&sb, &again, &oid);
    assert_eq!(answer["answer"], "declined");
    assert_eq!(answer["item"], first["item"]);
}

#[test]
fn adding_a_removal_of_a_task_the_owner_changed_or_finished_names_the_removal() {
    let _guard = support::guard(
        "adding_a_removal_of_a_task_the_owner_changed_or_finished_names_the_removal",
    );
    for (change, field) in [
        (
            &["edit", "", "--subject", "Call the dentist today"][..],
            "subject",
        ),
        (&["done", ""][..], "done"),
    ] {
        let sb = Sandbox::new();
        let held = grant_to(&sb, "tool", 1, vec![grant(&EVERY_RIGHT)]);
        let oid = added_from(&sb, &held, &["new", "Call the dentist"]);
        let item = removal(&sb, &held, &oid)["item"]
            .as_str()
            .unwrap()
            .to_string();
        let mut line = change.to_vec();
        line[1] = &oid;
        owner(&sb, &line);
        let out = sb.output(&["add", &item, "--json"]);
        assert_eq!(refused(&out), "item_removes_a_changed_task");
        let text = String::from_utf8_lossy(&sb.output(&["add", &item]).stderr).into_owned();
        assert!(
            text.contains(&format!(
                "{} removes {}, whose {field} you changed",
                &item[..7],
                &oid[..7]
            )),
            "{text}"
        );
    }
}
