use crate::nvim_fixtures::{answer, assert_fixture, error_document};
use crate::nvim_status_fixtures::without_ages;
use crate::support;

use support::fixture::{self, CALL_MOM, OAT_MILK, STANDUP, run, short};
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

const EXIT_REFUSED: i32 = 4;
const RIGHTS: [&str; 3] = ["task:create", "task:read", "task:change"];

fn planned(sb: &Sandbox, task: u8, priority: &str, label: &str, due: &str) {
    let task = short(task);
    let edit = [
        "edit",
        &task,
        "--priority",
        priority,
        "--labels",
        label,
        "--due",
        due,
    ];
    run(sb, &edit);
}

fn newest_commit(sb: &Sandbox) -> String {
    let log: serde_json::Value =
        serde_json::from_str(&answer(sb, &["log", "-o", "json"], "log.schema.json")).unwrap();
    log["commits"][0]["id"].as_str().unwrap().to_string()
}

fn sent(sb: &Sandbox, held: &str, args: &[&str]) {
    let mut line = args.to_vec();
    line.extend(["-o", "json"]);
    let out = sb.output_holding(held, &line);
    assert!(out.status.success(), "{args:?}: {out:?}");
}

#[test]
fn the_status_full_fixture_holds_each_rows_before_and_after() {
    let _guard = support::guard("the_status_full_fixture_holds_each_rows_before_and_after")
        .allow_slow(
            "it stages a change and has a token send an item and a set, one dam process each",
        );
    let sb = fixture::committed();
    planned(&sb, OAT_MILK, "4", "shop", "2026-09-28");
    run(&sb, &["add", "-A"]);
    planned(&sb, CALL_MOM, "1", "phone", "2026-09-30");
    run(&sb, &["rm", &short(STANDUP)]);
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS)]);
    sent(&sb, &held, &["new", "call the plumber"]);
    let both = [
        "edit",
        &short(CALL_MOM),
        &short(OAT_MILK),
        "--priority",
        "3",
        "--as-one",
    ];
    sent(&sb, &held, &both);
    let full = answer(
        &sb,
        &["status", "--full", "-o", "json"],
        "status.schema.json",
    );
    assert_fixture("status-full.json", &without_ages(&full));
}

#[test]
fn the_log_fixtures_are_what_dam_log_show_and_revert_answer() {
    let _guard = support::guard("the_log_fixtures_are_what_dam_log_show_and_revert_answer")
        .allow_slow("it makes three commits and reverts one, one dam process each");
    let sb = fixture::committed();
    planned(&sb, OAT_MILK, "4", "shop", "2026-09-28");
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "plan the errand"]);
    let errand = newest_commit(&sb);
    run(
        &sb,
        &["edit", &short(CALL_MOM), "--subject", "call mom back"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "call back"]);
    assert_fixture(
        "log.json",
        &answer(&sb, &["log", "-o", "json"], "log.schema.json"),
    );
    let shown = ["show", &errand, "-o", "json"];
    assert_fixture("show-commit.json", &answer(&sb, &shown, "show.schema.json"));
    run(&sb, &["edit", &short(OAT_MILK), "--priority", "3"]);
    let reverted = ["revert", &errand, "-o", "json"];
    assert_fixture("revert.json", &answer(&sb, &reverted, "revert.schema.json"));
}

#[test]
fn the_revert_refusal_fixture_is_what_dam_says_for_a_pulled_commit() {
    let _guard = support::guard("the_revert_refusal_fixture_is_what_dam_says_for_a_pulled_commit")
        .allow_slow("it pushes, changes the fake remote and pulls, one dam process each");
    let sb = fixture::committed();
    run(&sb, &["push"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    run(&sb, &["pull"]);
    let refused = sb.output(&["revert", &newest_commit(&sb), "-o", "json"]);
    assert_fixture(
        "error-revert-pulled.json",
        &error_document(refused, EXIT_REFUSED),
    );
}
