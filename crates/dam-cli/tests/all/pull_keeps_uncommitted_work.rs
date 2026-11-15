use crate::support;

use serde_json::Value;
use support::config_files::Remote;
use support::sandbox::Sandbox;
use support::withholding_helper::listed_task;

fn pushed_ann() -> (Sandbox, String) {
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["Ann"]);
    sb.commit_everything();
    let ann = sb.oid_of("Ann");
    sb.answer_the_next_push();
    sb.run(&["push"]);
    (sb, ann)
}

fn pull_ann_as(sb: &Sandbox, ann: &str, subject: &str) -> String {
    sb.answer_the_next_pull(&[listed_task(&format!("r-{ann}"), subject)], &[]);
    let (ok, out, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    out
}

fn subjects_here(sb: &Sandbox) -> Vec<String> {
    let (ok, out, err) = sb.dam(&["ls", "--json"]);
    assert!(ok, "{err}");
    let listed: Value = serde_json::from_str(&out).unwrap();
    let objects = listed["objects"].as_array().cloned().unwrap_or_default();
    let subject = |o: &Value| o["subject"].as_str().unwrap_or_default().to_string();
    objects.iter().map(subject).collect()
}

fn conflicts(sb: &Sandbox) -> Vec<Value> {
    let (ok, out, err) = sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: Value = serde_json::from_str(&out).unwrap();
    status["conflicts"].as_array().cloned().unwrap()
}

#[test]
fn a_pull_leaves_an_uncommitted_removal_alone_when_the_app_did_not_change_it() {
    let _guard =
        support::guard("a_pull_leaves_an_uncommitted_removal_alone_when_the_app_did_not_change_it");
    let (sb, ann) = pushed_ann();
    sb.run(&["rm", &ann[..7]]);

    pull_ann_as(&sb, &ann, "Ann");

    assert!(subjects_here(&sb).is_empty());
    assert!(conflicts(&sb).is_empty());
}

#[test]
fn a_pull_holds_an_uncommitted_removal_as_a_conflict_that_resolve_settles() {
    let _guard =
        support::guard("a_pull_holds_an_uncommitted_removal_as_a_conflict_that_resolve_settles");
    let (sb, ann) = pushed_ann();
    sb.run(&["rm", &ann[..7]]);

    let out = pull_ann_as(&sb, &ann, "Ann renamed");

    assert!(out.contains("1 conflict"), "{out}");
    assert!(subjects_here(&sb).is_empty());
    let held = conflicts(&sb);
    assert_eq!(held.len(), 1);
    assert_eq!(held[0]["ours"], Value::Null);
    assert_eq!(held[0]["theirs"]["subject"], "Ann renamed");
    let (_, status, _) = sb.dam(&["status"]);
    assert!(status.contains("ours: removed"), "{status}");
    assert!(
        status.contains(&format!("dam resolve {ann} --ours|--theirs")),
        "{status}"
    );

    sb.run(&["resolve", &ann, "--ours"]);

    assert!(conflicts(&sb).is_empty());
    assert!(subjects_here(&sb).is_empty());
}

#[test]
fn a_pull_holds_an_uncommitted_edit_as_a_conflict_that_resolve_settles() {
    let _guard =
        support::guard("a_pull_holds_an_uncommitted_edit_as_a_conflict_that_resolve_settles");
    let (sb, ann) = pushed_ann();
    sb.run(&["edit", &ann[..7], "--subject", "Ann here"]);

    pull_ann_as(&sb, &ann, "Ann there");

    assert_eq!(subjects_here(&sb), vec!["Ann here".to_string()]);
    let held = conflicts(&sb);
    assert_eq!(held[0]["ours"]["subject"], "Ann here");

    sb.run(&["resolve", &ann[..7], "--theirs"]);

    assert!(conflicts(&sb).is_empty());
    assert_eq!(subjects_here(&sb), vec!["Ann there".to_string()]);
}
