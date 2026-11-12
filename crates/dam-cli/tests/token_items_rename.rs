mod support;

use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to, refused};

fn dam(sb: &Sandbox, line: &[&str]) {
    let (ok, _, err) = sb.dam(line);
    assert!(ok, "{err}");
}

fn path_of(sb: &Sandbox, oid: &str) -> String {
    let (ok, out, err) = sb.dam(&["show", &oid[..7], "--json"]);
    assert!(ok, "{err}");
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    json["path"].as_str().unwrap().to_string()
}

fn renamed_by_vpt(sb: &Sandbox, oid: &str, subject: &str) -> String {
    let held = grant_to(sb, "vpt", 1, &grant("\"task:read\", \"task:change\""));
    let line = ["edit", oid, "--subject", subject, "--json"];
    let sent = answer(&sb.output_holding(&held, &line), "sent.schema.json");
    sent["hash"].as_str().unwrap().to_string()
}

fn committed(sb: &Sandbox) {
    dam(sb, &["add", "-A"]);
    dam(sb, &["commit", "-m", "seed"]);
}

#[test]
fn adding_a_tokens_rename_carries_the_tasks_children() {
    let _guard = support::guard("adding_a_tokens_rename_carries_the_tasks_children");
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    let child = sb.new_object(&["child", "--path", "parent/"]);
    let grandchild = sb.new_object(&["grandchild", "--path", "parent/child/"]);
    committed(&sb);
    let hash = renamed_by_vpt(&sb, &parent, "renamed");

    dam(&sb, &["add", &hash]);

    assert_eq!(path_of(&sb, &child), "renamed/");
    assert_eq!(path_of(&sb, &grandchild), "renamed/child/");
}

#[test]
fn adding_a_tokens_rename_that_cannot_name_a_folder_is_refused_once_the_task_has_children() {
    let _guard = support::guard(
        "adding_a_tokens_rename_that_cannot_name_a_folder_is_refused_once_the_task_has_children",
    );
    let sb = Sandbox::new();
    let parent = sb.new_object(&["parent"]);
    committed(&sb);
    let hash = renamed_by_vpt(&sb, &parent, ".");
    let child = sb.new_object(&["child", "--path", "parent/"]);

    assert_eq!(
        refused(&sb.output(&["add", &hash, "--json"])),
        "item_subject_names_no_folder"
    );
    assert_eq!(path_of(&sb, &child), "parent/");
    let (_, status, _) = sb.dam(&["status", "--json"]);
    let status: serde_json::Value = serde_json::from_str(&status).unwrap();
    assert_eq!(status["waiting"][0]["hash"], hash.as_str());
}
