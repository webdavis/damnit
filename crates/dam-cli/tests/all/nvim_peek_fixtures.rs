use crate::nvim_fixtures::{answer, assert_fixture, fed_answer};
use crate::support;

use support::fixture::{self, CALL_MOM, OAT_MILK};

fn twenty_lines() -> String {
    (1..=20)
        .map(|n| format!("step {n} of the plan"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_related_fixtures_are_what_dam_answers_for_a_task_its_parent_and_its_children() {
    let _guard = support::guard(
        "the_related_fixtures_are_what_dam_answers_for_a_task_its_parent_and_its_children",
    );
    let sb = fixture::seeded();
    let body = twenty_lines();
    let task = sb.new_object(&[
        "plan the garden",
        "--parent",
        &fixture::oid(CALL_MOM),
        "--depends",
        &fixture::oid(OAT_MILK),
        "--due",
        "2026-09-25",
        "--deadline",
        "2026-09-30",
        "--priority",
        "2",
        "--labels",
        "garden",
        "--category",
        "effort=deep",
        "--estimate",
        "45",
        "--recurrence",
        "every week",
        "--body",
        &body,
    ]);
    sb.new_object(&["dig the beds", "--parent", &task]);
    sb.new_object(&["buy the seeds", "--parent", &task]);
    let shown = answer(&sb, &["show", &task, "-o", "json"], "show.schema.json");
    assert_fixture("show-related.json", &shown);
    let oid = serde_json::from_str::<serde_json::Value>(&shown).unwrap()["oid"].clone();
    let request = serde_json::json!({ "parent": oid }).to_string();
    let sent = ["list", "--request", "-", "-o", "json"];
    assert_fixture("list-children.json", &fed_answer(&sb, &sent, &request));
}
