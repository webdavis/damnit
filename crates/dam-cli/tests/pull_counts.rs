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

fn a_pull_that_uses_every_rule() -> Sandbox {
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());
    sb.install_helper_withholding_answers();
    sb.new_object(&["call Bob"]);
    sb.new_object(&["call tool"]);
    commit(&sb);
    let (bob, tool) = (oid_of(&sb, "call Bob"), oid_of(&sb, "call tool"));
    sb.answer_the_next_push();
    run(&sb, &["push"]);

    run(&sb, &["edit", &bob[..7], "--subject", "call Bob at 5"]);
    run(&sb, &["rm", &tool[..7]]);
    sb.new_object(&["call Ada"]);
    commit(&sb);
    sb.dam(&["push"]);
    run(&sb, &["edit", &bob[..7], "--subject", "call Bob at 6"]);
    commit(&sb);

    let objects = [
        listed_task("r-new", "call Ada"),
        listed_task(&format!("r-{bob}"), "call Bob at 5"),
    ];
    sb.answer_the_next_pull(&objects, &[format!("r-{tool}")]);
    sb
}

#[test]
fn a_pull_prints_what_it_took_from_the_record_of_attempts() {
    let _guard = support::guard("a_pull_prints_what_it_took_from_the_record_of_attempts");
    let sb = a_pull_that_uses_every_rule();

    let (ok, out, err) = sb.dam(&["pull"]);

    assert!(ok, "{err}");
    assert_eq!(
        out,
        "fake: 0 new, 0 updated, 2 unchanged, 0 conflicts, 0 removed upstream\n  \
         1 create matched\n  1 value kept\n  1 delete confirmed\n"
    );
}

#[test]
fn a_pull_answers_what_it_took_from_the_record_of_attempts() {
    let _guard = support::guard("a_pull_answers_what_it_took_from_the_record_of_attempts");
    let sb = a_pull_that_uses_every_rule();

    let (ok, out, err) = sb.dam(&["--json", "pull"]);

    assert!(ok, "{err}");
    let answer: Value = serde_json::from_str(&out).unwrap();
    support::schema::assert_matches_schema(&answer, "pull.schema.json");
    let row = &answer["remotes"][0];
    let counts = ["creates_matched", "values_kept", "deletes_confirmed"].map(|k| &row[k]);
    assert_eq!(counts, [1, 1, 1], "{answer}");
}

#[test]
fn a_pull_that_took_nothing_from_the_record_prints_one_line() {
    let _guard = support::guard("a_pull_that_took_nothing_from_the_record_prints_one_line");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake());

    let (ok, out, err) = sb.dam(&["pull"]);

    assert!(ok, "{err}");
    assert_eq!(out.lines().count(), 1, "{out}");
}
