mod support;

use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;

const REMINDED_UPSTREAM: &str = r#"{"oid":"","remote_id":"up-1","kind":"task","subject":"call the bank","reminders":[30,1440],"task":{"done":false,"priority":4,"due":"2026-01-02T10:00:00+00:00[UTC]"}}"#;

#[test]
fn reminders_list_the_ones_that_fall_due_in_the_window_as_json_and_toon() {
    let _guard =
        support::guard("reminders_list_the_ones_that_fall_due_in_the_window_as_json_and_toon");
    let sb = Sandbox::new();
    sb.install_helper_pulling(REMINDED_UPSTREAM);
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");

    let window = ["--from", "2026-01-02T09:00", "--to", "2026-01-02T10:00"];
    let mut args = vec!["reminders", "--no-pull", "--json"];
    args.extend(window);
    let (ok, out, err) = sb.dam(&args);
    assert!(ok, "{err}");
    let due: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_matches_schema(&due, "reminders.schema.json");
    let reminders = due["reminders"].as_array().unwrap();
    assert_eq!(reminders.len(), 1, "{due}");
    let at = reminders[0]["at"]
        .as_str()
        .unwrap()
        .parse::<jiff::Timestamp>();
    assert_eq!(at.unwrap(), "2026-01-02T09:30:00Z".parse().unwrap());
    assert_eq!(reminders[0]["minutes_before"], 30);
    assert_eq!(reminders[0]["subject"], "call the bank");

    args[2] = "--toon";
    let (ok, out, err) = sb.dam(&args);
    assert!(ok, "{err}");
    assert!(out.starts_with("reminders[1]{"), "{out}");
}
