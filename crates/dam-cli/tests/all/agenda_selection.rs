use crate::support;

use support::fixture;

#[test]
fn agenda_narrows_by_a_selection_flag_over_a_whole_day_given_as_to() {
    let _guard = support::guard("agenda_narrows_by_a_selection_flag_over_a_whole_day_given_as_to");
    let sb = fixture::committed();
    let line = [
        "agenda",
        "--from",
        "2026-09-25",
        "--to",
        "2026-09-25",
        "--subject",
        "standup",
        "-o",
        "json",
    ];
    let out = sb.output(&line);
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let subjects: Vec<&str> = answer["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects, ["standup"]);
}
