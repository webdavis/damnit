mod support;

use support::hidden::{Hidden, full_oid};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn committed_task_from_tool() -> Hidden {
    let sb = Sandbox::new();
    let held = grant_to(
        &sb,
        "tool",
        1,
        vec![grant(&["task:read", "task:create"]).paths(&["work"])],
    );
    let line = ["new", "Call the dentist", "--path", "work", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    let dentist = sent["oid"].as_str().unwrap().to_string();
    let h = Hidden {
        sb,
        held,
        report: dentist,
    };
    h.owner(&["add", "-A"]);
    h.owner(&["commit", "-m", "tool work"]);
    h
}

#[test]
fn the_owner_restores_a_removed_task_that_came_from_a_tool() {
    let _guard = support::guard("the_owner_restores_a_removed_task_that_came_from_a_tool");
    let h = committed_task_from_tool();
    h.owner(&["rm", &h.report]);
    h.owner(&["restore", &h.report]);
    assert_eq!(full_oid(&h.sb, &h.report[..7]), h.report);
}

#[test]
fn the_owner_stages_and_commits_the_removal_of_a_task_that_came_from_a_tool() {
    let _guard =
        support::guard("the_owner_stages_and_commits_the_removal_of_a_task_that_came_from_a_tool");
    let h = committed_task_from_tool();
    h.owner(&["rm", &h.report]);
    h.owner(&["add", &h.report]);
    h.owner(&["commit", "-m", "gone"]);
    let (ok, out, err) = h.sb.dam(&["log", "--json"]);
    assert!(ok, "{err}");
    let log: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(log["commits"][0]["changes"][0]["op"], "delete");
}

#[test]
fn a_token_cannot_tell_its_removed_task_from_one_moved_out_of_its_grants() {
    let _guard =
        support::guard("a_token_cannot_tell_its_removed_task_from_one_moved_out_of_its_grants");
    let h = committed_task_from_tool();
    h.owner(&["mv", &h.report, "home"]);
    h.answers_alike_once_removed(&h.report, &["show", &h.report, "--json"]);
}
