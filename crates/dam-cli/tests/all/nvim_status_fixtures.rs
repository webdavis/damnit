use crate::nvim_fixtures::{answer, assert_fixture};
use crate::support;

use support::fixture::{self, CALL_MOM, OAT_MILK, oid, run, short};
use support::sandbox::Sandbox;
use support::tokens::{grant, grant_to};

const RIGHTS: [&str; 3] = ["task:create", "task:read", "task:change"];

pub(crate) fn without_ages(text: &str) -> String {
    text.lines()
        .map(|line| match line.find("\"age\": ") {
            Some(at) => {
                let comma = if line.ends_with(',') { "," } else { "" };
                format!("{}\"age\": 0{comma}", &line[..at])
            }
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn sent(sb: &Sandbox, held: &str, args: &[&str]) -> serde_json::Value {
    let mut line = args.to_vec();
    line.extend(["-o", "json"]);
    let out = sb.output_holding(held, &line);
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

fn in_conflict() -> Sandbox {
    let sb = fixture::committed();
    run(&sb, &["push"]);
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "switch milk"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    run(&sb, &["pull"]);
    sb
}

struct Waiting {
    held: String,
    plumber: String,
    set: String,
}

fn with_every_section() -> (Sandbox, Waiting) {
    let sb = in_conflict();
    let rent = sb.new_object(&["pay rent"]);
    run(&sb, &["add", &rent]);
    run(&sb, &["commit", "-m", "rent"]);
    let held = grant_to(&sb, "tool", 1, vec![grant(&RIGHTS)]);
    let plumber = sent(&sb, &held, &["new", "call the plumber"]);
    let edits = [
        "edit",
        &short(CALL_MOM),
        &rent,
        "--priority",
        "1",
        "--as-one",
    ];
    let set = sent(&sb, &held, &edits);
    run(
        &sb,
        &["edit", &short(CALL_MOM), "--subject", "call mom back"],
    );
    let plants = sb.new_object(&["water the plants"]);
    run(&sb, &["add", &plants]);
    let waiting = Waiting {
        plumber: plumber["hash"].as_str().unwrap().to_string(),
        set: set["set"].as_str().unwrap().to_string(),
        held,
    };
    (sb, waiting)
}

fn status(sb: &Sandbox) -> String {
    without_ages(&answer(sb, &["status", "-o", "json"], "status.schema.json"))
}

#[test]
fn the_status_fixtures_are_what_dam_status_answers() {
    let _guard = support::guard("the_status_fixtures_are_what_dam_status_answers")
        .allow_slow("it builds a conflict, a token's items and a set, one dam process each");
    assert_fixture("status-empty.json", &status(&Sandbox::new()));
    let (sb, _) = with_every_section();
    assert_fixture("status.json", &status(&sb));
}

#[test]
fn the_status_action_fixtures_are_what_each_key_sends() {
    let _guard = support::guard("the_status_action_fixtures_are_what_each_key_sends")
        .allow_slow("it runs every command the status window sends, one dam process each");
    let (sb, waiting) = with_every_section();
    let mom = oid(CALL_MOM);
    let add = answer(&sb, &["add", &mom, "-o", "json"], "add.schema.json");
    assert_fixture("status-add.json", &add);
    let reset = answer(&sb, &["reset", &mom, "-o", "json"], "reset.schema.json");
    assert_fixture("status-reset.json", &reset);
    let accept = ["add", &waiting.plumber, "-o", "json"];
    assert_fixture(
        "status-accept.json",
        &answer(&sb, &accept, "add.schema.json"),
    );
    let tiles = sent(&sb, &waiting.held, &["new", "order the tiles"]);
    let decline = ["rm", tiles["hash"].as_str().unwrap(), "-o", "json"];
    assert_fixture(
        "status-decline.json",
        &answer(&sb, &decline, "settled.schema.json"),
    );
    let decline_set = ["rm", &waiting.set, "-o", "json"];
    let declined = answer(&sb, &decline_set, "settled-set.schema.json");
    assert_fixture("status-decline-set.json", &declined);
    answer(&sb, &["add", "-A", "-o", "json"], "add.schema.json");
    answer(&sb, &["reset", "-o", "json"], "reset.schema.json");
    answer(&sb, &["add", &mom, "-o", "json"], "add.schema.json");
    let commit = ["commit", "-m", "plugin work", "-o", "json"];
    assert_fixture(
        "status-commit.json",
        &answer(&sb, &commit, "commit.schema.json"),
    );
}

#[test]
fn the_push_and_pull_fixtures_are_what_dam_answers_from_the_fake_remote() {
    let _guard =
        support::guard("the_push_and_pull_fixtures_are_what_dam_answers_from_the_fake_remote");
    let sb = fixture::committed();
    assert_fixture(
        "pull.json",
        &answer(&sb, &["pull", "-o", "json"], "pull.schema.json"),
    );
    assert_fixture(
        "push.json",
        &answer(&sb, &["push", "-o", "json"], "push.schema.json"),
    );
    answer(&sb, &["pull", "fake", "-o", "json"], "pull.schema.json");
    answer(&sb, &["push", "fake", "-o", "json"], "push.schema.json");
}
