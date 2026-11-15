use crate::support;

use serde_json::Value;
use support::fixture::{self, CALL_MOM, OAT_MILK, oid, run, short};
use support::sandbox::Sandbox;
use support::tokens::{answer, grant, grant_to};

fn milk_in_conflict_and_mom_renamed() -> Sandbox {
    let sb = fixture::committed();
    run(&sb, &["push"]);
    run(
        &sb,
        &["edit", &short(OAT_MILK), "--subject", "buy soy milk"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "switch milk"]);
    let kept = sb.dir.path().join("remote.kept");
    let mut theirs: Value = serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
    theirs["subject"] = "buy rice milk".into();
    std::fs::write(&kept, theirs.to_string()).unwrap();
    run(&sb, &["pull"]);
    run(
        &sb,
        &["edit", &short(CALL_MOM), "--subject", "call mom at 5"],
    );
    run(&sb, &["add", "-A"]);
    run(&sb, &["commit", "-m", "call later"]);
    sb
}

fn oids_in_the_last_push(sb: &Sandbox) -> Vec<String> {
    let pushes = sb.pushes();
    let last = pushes.last().unwrap()["mutations"]
        .as_array()
        .cloned()
        .unwrap();
    last.iter()
        .map(|m| m["oid"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_push_sends_the_rest_and_holds_the_object_in_conflict() {
    let _guard = support::guard("a_push_sends_the_rest_and_holds_the_object_in_conflict");
    let sb = milk_in_conflict_and_mom_renamed();

    let (ok, out, err) = sb.dam(&["push"]);

    assert!(ok, "{err}");
    assert_eq!(oids_in_the_last_push(&sb), vec![oid(CALL_MOM)]);
    assert_eq!(
        out,
        "fake: 1 sent, 1 ok, 0 failed, 0 skipped\n  \
         1 held for a conflict; see dam status, settle with dam resolve\n"
    );
}

#[test]
fn a_push_answers_how_many_it_held() {
    let _guard = support::guard("a_push_answers_how_many_it_held");
    let sb = milk_in_conflict_and_mom_renamed();

    let document = answer(&sb.output(&["push", "--json"]), "push.schema.json");

    assert_eq!(document["remotes"][0]["held"], 1, "{document}");
}

#[test]
fn a_token_without_conflict_read_is_not_told_how_many_were_held() {
    let _guard = support::guard("a_token_without_conflict_read_is_not_told_how_many_were_held");
    let sb = milk_in_conflict_and_mom_renamed();
    let held = grant_to(&sb, "tool", 3, vec![grant(&["remote:push"])]);

    let out = sb.output_holding(&held, &["push", "--json"]);

    let document = answer(&out, "push.schema.json");
    assert_eq!(document["remotes"][0]["held"], Value::Null, "{document}");
}

#[test]
fn the_held_object_goes_out_on_the_push_after_resolve() {
    let _guard = support::guard("the_held_object_goes_out_on_the_push_after_resolve");
    let sb = milk_in_conflict_and_mom_renamed();
    run(&sb, &["push"]);

    run(&sb, &["resolve", &short(OAT_MILK), "--ours"]);
    let (ok, out, err) = sb.dam(&["push"]);

    assert!(ok, "{err}");
    assert_eq!(oids_in_the_last_push(&sb), vec![oid(OAT_MILK)]);
    assert_eq!(out, "fake: 1 sent, 1 ok, 0 failed, 0 skipped\n");
}
