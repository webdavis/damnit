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

#[test]
fn agenda_kind_any_lists_the_open_tasks_due_in_the_window_beside_the_events() {
    let _guard =
        support::guard("agenda_kind_any_lists_the_open_tasks_due_in_the_window_beside_the_events");
    let sb = fixture::committed();
    let line = [
        "agenda",
        "--kind",
        "any",
        "--from",
        "2026-09-25",
        "--to",
        "2026-09-25",
    ];
    let out = sb.output(&[&line[..], &["-o", "json"]].concat());
    assert!(out.status.success(), "{out:?}");
    let answer: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let subjects = |key: &str| -> Vec<String> {
        let rows = answer[key].as_array().unwrap();
        rows.iter()
            .map(|r| r["subject"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(subjects("events"), ["offsite", "standup"]);
    assert_eq!(subjects("tasks"), ["buy oat milk"]);
}
