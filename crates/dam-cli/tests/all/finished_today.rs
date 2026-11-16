use crate::support;

use support::sandbox::Sandbox;

fn subjects(sb: &Sandbox, flags: &[&str]) -> Vec<String> {
    let (ok, out, err) = sb.dam(&[&["list"], flags, &["-o", "json"]].concat());
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn completed_today_finds_the_tasks_finished_today_by_their_completion_time() {
    let _guard =
        support::guard("completed_today_finds_the_tasks_finished_today_by_their_completion_time");
    let sb = Sandbox::new();
    let finished = sb.new_object(&["water the plants"]);
    sb.new_object(&["still open"]);
    assert!(sb.dam(&["done", &finished]).0);

    assert_eq!(
        subjects(&sb, &["--completed-at", "today"]),
        vec!["water the plants"]
    );
    assert!(subjects(&sb, &["--completed-at", "today-1"]).is_empty());
    assert_eq!(
        subjects(&sb, &["--completed-at", "none"]),
        vec!["still open"]
    );
}
