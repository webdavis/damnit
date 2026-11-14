mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn objects(sb: &Sandbox) -> Vec<serde_json::Value> {
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).unwrap()["objects"]
        .as_array()
        .unwrap()
        .clone()
}

fn owner_new(sb: &Sandbox, args: &[&str]) -> serde_json::Value {
    let mut line = vec!["new"];
    line.extend_from_slice(args);
    line.push("--json");
    let (ok, out, err) = sb.dam(&line);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

#[test]
fn the_owners_repeat_with_the_same_id_changes_that_task() {
    let _guard = support::guard("the_owners_repeat_with_the_same_id_changes_that_task");
    let sb = Sandbox::new();
    let first = owner_new(&sb, &["Call dentist", "--id", "dentist"]);
    let again = owner_new(&sb, &["Call the dentist", "--id", "dentist", "-p", "1"]);
    assert_eq!(again["oid"], first["oid"]);
    let all = objects(&sb);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["subject"], "Call the dentist");
    assert_eq!(all[0]["task"]["priority"], 1);
}

#[test]
fn different_ids_make_different_tasks() {
    let _guard = support::guard("different_ids_make_different_tasks");
    let sb = Sandbox::new();
    let a = owner_new(&sb, &["Call dentist", "--id", "a"]);
    let b = owner_new(&sb, &["Call dentist", "--id", "b"]);
    assert_ne!(a["oid"], b["oid"]);
    assert_eq!(objects(&sb).len(), 2);
}

#[test]
fn the_owners_ids_are_kept_apart_from_a_tokens() {
    let _guard = support::guard("the_owners_ids_are_kept_apart_from_a_tokens");
    let sb = Sandbox::new();
    let held = grant_to(&sb, "tool", 1, vec![grant(&["task:read", "task:create"])]);
    let mine = owner_new(&sb, &["Call dentist", "--id", "dentist"]);
    let line = ["new", "Call the dentist", "--id", "dentist", "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    assert_eq!(sent["answer"], "new");
    assert_ne!(sent["oid"], mine["oid"]);
    assert_eq!(objects(&sb)[0]["subject"], "Call dentist");
}

#[test]
fn an_id_with_a_line_break_or_past_the_cap_is_a_usage_error() {
    let _guard = support::guard("an_id_with_a_line_break_or_past_the_cap_is_a_usage_error");
    let sb = Sandbox::new();
    let long = "x".repeat(201);
    for bad in ["a\nb", "a\rb", "", long.as_str()] {
        let out = sb.output(&["new", "Call dentist", "--id", bad]);
        assert_eq!(out.status.code(), Some(2), "{bad:?}: {out:?}");
    }
    assert!(objects(&sb).is_empty());
    let fits = "x".repeat(200);
    owner_new(&sb, &["Call dentist", "--id", &fits]);
}

#[test]
fn the_help_says_the_id_is_not_damnits_oid() {
    let _guard = support::guard("the_help_says_the_id_is_not_damnits_oid");
    let sb = Sandbox::new();
    let (ok, out, err) = sb.dam(&["new", "--help"]);
    assert!(ok, "{err}");
    let help = out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(help.contains("--id"), "{out}");
    assert!(help.contains("not damnit's oid"), "{out}");
}

#[test]
fn the_owners_repeat_that_moves_the_task_carries_its_children_as_mv_does() {
    let _guard =
        support::guard("the_owners_repeat_that_moves_the_task_carries_its_children_as_mv_does");
    let sb = Sandbox::new();
    owner_new(&sb, &["Plan", "--id", "plan", "--path", "work"]);
    let step = owner_new(&sb, &["Step", "--path", "work/Plan"]);
    owner_new(&sb, &["Plan", "--id", "plan", "--path", "home"]);
    let all = objects(&sb);
    let moved = all.iter().find(|o| o["oid"] == step["oid"]).unwrap();
    assert_eq!(moved["path"], "home/Plan/");
}

#[test]
fn the_owners_repeat_into_its_own_folder_is_refused_as_mv_is() {
    let _guard = support::guard("the_owners_repeat_into_its_own_folder_is_refused_as_mv_is");
    let sb = Sandbox::new();
    owner_new(&sb, &["Plan", "--id", "plan", "--path", "work"]);
    let line = [
        "new",
        "Plan",
        "--id",
        "plan",
        "--path",
        "work/Plan/x",
        "--json",
    ];
    assert_eq!(refused(&sb.output(&line)), "move_inside_itself");
    assert_eq!(objects(&sb)[0]["path"], "work/");
}

#[test]
fn the_owners_repeat_that_renames_the_task_carries_its_children_as_edit_does() {
    let _guard =
        support::guard("the_owners_repeat_that_renames_the_task_carries_its_children_as_edit_does");
    let sb = Sandbox::new();
    owner_new(&sb, &["Plan", "--id", "plan", "--path", "work"]);
    let step = owner_new(&sb, &["Step", "--path", "work/Plan"]);
    let detail = owner_new(&sb, &["Detail", "--path", "work/Plan/Step"]);
    owner_new(&sb, &["Roadmap", "--id", "plan", "--path", "home"]);
    let all = objects(&sb);
    let path_of =
        |oid: &serde_json::Value| all.iter().find(|o| o["oid"] == *oid).unwrap()["path"].clone();
    assert_eq!(path_of(&step["oid"]), "home/Roadmap/");
    assert_eq!(path_of(&detail["oid"]), "home/Roadmap/Step/");
}

#[test]
fn the_answer_and_show_print_the_callers_id_under_id() {
    let _guard = support::guard("the_answer_and_show_print_the_callers_id_under_id");
    let sb = Sandbox::new();
    let made = owner_new(&sb, &["Call dentist", "--id", "dentist"]);
    assert_eq!(made["id"], "dentist");
    let oid = made["oid"].as_str().unwrap();
    let (ok, out, err) = sb.dam(&["show", oid, "--json"]);
    assert!(ok, "{err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["id"],
        "dentist"
    );
    let plain = owner_new(&sb, &["Call vet"]);
    assert!(plain.get("id").is_none(), "{plain}");
}
