mod support;

use support::sandbox::Sandbox;

const PULLED_EVERY_OTHER_WEEK: &str = r#"{"oid":"","remote_id":"up-1","kind":"task","subject":"water the plants","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":"every other week","task":{"done":false,"priority":2,"due":"2026-10-01","deadline":null,"event":null},"event":null}"#;

#[test]
fn done_refuses_a_pulled_rule_dam_cannot_read_and_the_task_stays_open() {
    let _guard =
        support::guard("done_refuses_a_pulled_rule_dam_cannot_read_and_the_task_stays_open");
    let sb = Sandbox::new();
    sb.install_helper_pulling(PULLED_EVERY_OTHER_WEEK);
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let listed: serde_json::Value = serde_json::from_str(&out).unwrap();
    let oid = listed["objects"][0]["oid"].as_str().unwrap().to_string();

    let refused = sb.output(&["done", &oid]);
    assert_eq!(refused.status.code(), Some(4));
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        "dam: cannot roll \"every other week\" forward; complete it in Todoist or set --recurrence\n"
    );
    let refused = sb.output(&["done", &oid, "--json"]);
    let document: serde_json::Value = serde_json::from_slice(&refused.stderr).unwrap();
    assert_eq!(document["error"]["rule"], "unreadable_recurrence");

    let (ok, out, _) = sb.dam(&["show", &oid, "--json", "--no-pull"]);
    assert!(ok);
    let shown: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(shown["task"]["done"], false, "{shown}");
}

#[test]
fn a_rule_the_stepper_cannot_serve_is_refused_at_the_edit_with_one_plain_line() {
    let _guard = support::guard(
        "a_rule_the_stepper_cannot_serve_is_refused_at_the_edit_with_one_plain_line",
    );
    let sb = Sandbox::new();
    let oid = sb.new_object(&["probe"]);
    for (rule, said) in [
        (
            "every day on mon",
            "dam: on mon names weekdays, which only a weekly rule takes; try every week on mon\n",
        ),
        (
            "every week on 5",
            "dam: on 5 names days of the month, which only a monthly rule takes; try every month on 5\n",
        ),
        (
            "every 210 weeks on mon",
            "dam: interval must be 1 to 208 weeks when weekdays are named, got \"210\"\n",
        ),
    ] {
        let out = sb.output(&["edit", &oid, "--recurrence", rule]);
        assert_eq!(out.status.code(), Some(1), "{rule}");
        assert_eq!(String::from_utf8_lossy(&out.stderr), said);
    }
}
