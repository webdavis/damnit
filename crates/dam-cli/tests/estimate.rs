mod support;

use support::sandbox::Sandbox;

fn subjects(sb: &Sandbox, query: &str) -> Vec<String> {
    let (ok, out, err) = sb.dam(&["ls", query, "--json"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect()
}

fn estimate(sb: &Sandbox, oid: &str) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["show", oid, "--json"]);
    assert!(ok, "{err}");
    serde_json::from_str::<serde_json::Value>(&out).unwrap()["task"]["estimate"].clone()
}

#[test]
fn an_estimate_in_minutes_finds_the_tasks_that_fit_five_minutes() {
    let _guard = support::guard("an_estimate_in_minutes_finds_the_tasks_that_fit_five_minutes");
    let sb = Sandbox::new();
    let quick = sb.new_object(&["water the plants"]);
    let long = sb.new_object(&["file taxes"]);
    sb.new_object(&["unestimated"]);
    assert!(sb.dam(&["edit", &quick, "--estimate", "5"]).0);
    assert!(sb.dam(&["edit", &long, "--estimate", "2h"]).0);

    assert_eq!(estimate(&sb, &quick), 5);
    assert_eq!(estimate(&sb, &long), 120);
    assert_eq!(subjects(&sb, "estimate:max:5m"), vec!["water the plants"]);
    assert_eq!(subjects(&sb, "estimate:min:1h"), vec!["file taxes"]);
    assert_eq!(subjects(&sb, "estimate:none"), vec!["unestimated"]);

    assert!(sb.dam(&["edit", &long, "--no-estimate"]).0);
    assert_eq!(estimate(&sb, &long), serde_json::Value::Null);
}

#[test]
fn an_estimate_that_is_not_whole_minutes_is_refused_as_usage() {
    let _guard = support::guard("an_estimate_that_is_not_whole_minutes_is_refused_as_usage");
    let sb = Sandbox::new();
    let oid = sb.new_object(&["x"]);
    let out = sb.output(&["edit", &oid, "--estimate", "90s"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}
