mod support;

use serde_json::Value;
use support::config_files::Remote;
use support::sandbox::Sandbox;
use support::withholding_helper::listed_task;

fn run(sb: &Sandbox, args: &[&str]) {
    let (ok, _, err) = sb.dam(args);
    assert!(ok, "{args:?}: {err}");
}

fn oid_of(sb: &Sandbox, subject: &str) -> String {
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let listed: Value = serde_json::from_str(&out).unwrap();
    let objects = listed["objects"].as_array().unwrap();
    let object = objects.iter().find(|o| o["subject"] == subject).unwrap();
    object["oid"].as_str().unwrap().to_string()
}

fn commit(sb: &Sandbox) {
    run(sb, &["add", "-A"]);
    run(sb, &["commit", "-m", "m"]);
}

fn mutations_of_the_last_push(sb: &Sandbox) -> Vec<(String, String, String)> {
    let pushes = sb.pushes();
    let last = pushes.last().unwrap();
    let mutations = last["mutations"].as_array().unwrap();
    mutations
        .iter()
        .map(|m| {
            let field = |k: &str| m[k].as_str().unwrap_or_default().to_string();
            (
                field("op"),
                field("oid"),
                m["object"]["subject"].to_string(),
            )
        })
        .collect()
}

#[test]
fn a_task_removed_in_the_app_stays_removed_after_a_push_that_leaves_its_commit_unpushed() {
    let _guard = support::guard(
        "a_task_removed_in_the_app_stays_removed_after_a_push_that_leaves_its_commit_unpushed",
    );
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["Ann"]);
    sb.new_object(&["Bob"]);
    commit(&sb);
    let (ann, bob) = (oid_of(&sb, "Ann"), oid_of(&sb, "Bob"));
    sb.answer_the_next_push();
    run(&sb, &["push"]);
    run(&sb, &["edit", &ann[..7], "--subject", "Ann 2"]);
    run(&sb, &["edit", &bob[..7], "--subject", "Bob 2"]);
    commit(&sb);
    sb.dam(&["push"]);
    sb.answer_the_next_pull(
        &[listed_task(&format!("r-{bob}"), "Bob")],
        &[format!("r-{ann}")],
    );
    run(&sb, &["pull"]);
    sb.dam(&["push"]);

    sb.answer_the_next_push();
    run(&sb, &["push"]);

    let bob_renamed = ("update".to_string(), bob, "\"Bob 2\"".to_string());
    assert_eq!(mutations_of_the_last_push(&sb), vec![bob_renamed]);
}
