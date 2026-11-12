mod support;

use support::sandbox::Sandbox;

fn json(sb: &Sandbox, args: &[&str]) -> serde_json::Value {
    let (ok, out, err) = sb.dam(args);
    assert!(ok, "{err}");
    serde_json::from_str(&out).unwrap()
}

fn subjects(listed: &serde_json::Value) -> Vec<&str> {
    listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap())
        .collect()
}

#[test]
fn three_focus_tasks_lead_the_list_in_the_order_they_were_marked_and_a_fourth_is_refused() {
    let _guard = support::guard(
        "three_focus_tasks_lead_the_list_in_the_order_they_were_marked_and_a_fourth_is_refused",
    );
    let sb = Sandbox::new();
    let oids: Vec<String> = ["a later task", "taxes", "dentist", "gym"]
        .iter()
        .map(|s| sb.new_object(&[s]))
        .collect();
    for oid in [&oids[2], &oids[1], &oids[3]] {
        assert!(sb.dam(&["edit", oid, "--focus", "today"]).0);
    }

    let fourth = sb.output(&["edit", &oids[0], "--focus", "today", "--json"]);
    assert_eq!(fourth.status.code(), Some(4));
    let error: serde_json::Value = serde_json::from_slice(&fourth.stderr).unwrap();
    assert_eq!(error["error"]["rule"], "focus_full", "{error}");
    support::schema::assert_matches_schema(&error, "error.schema.json");

    let focused = json(&sb, &["ls", "focus:today", "--json"]);
    assert_eq!(subjects(&focused), vec!["dentist", "taxes", "gym"]);
    assert_eq!(subjects(&json(&sb, &["ls", "--json"]))[0], "dentist");

    let mark = json(&sb, &["show", &oids[2], "--json"])["task"]["focus"].clone();
    let today = jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .date()
        .to_string();
    assert_eq!(mark["day"], today.as_str(), "{mark}");
    assert_eq!(mark["set_by"], "owner", "{mark}");
    mark["set_at"]
        .as_str()
        .unwrap()
        .parse::<jiff::Timestamp>()
        .unwrap();

    assert!(sb.dam(&["edit", &oids[1], "--no-focus"]).0);
    assert!(sb.dam(&["edit", &oids[0], "--focus", "today"]).0);
    assert!(sb.dam(&["edit", &oids[1], "--focus", "tomorrow"]).0);
    assert_eq!(
        subjects(&json(&sb, &["ls", "focus:today", "--json"])),
        vec!["dentist", "gym", "a later task"]
    );
}
