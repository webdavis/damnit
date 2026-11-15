use crate::support;

use support::errors::error_document;
use support::sandbox::Sandbox;

fn busy_intervals(document: &str) -> Vec<(String, String, bool)> {
    let value: serde_json::Value = serde_json::from_str(document).unwrap();
    value["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["start"].as_str().unwrap().to_string(),
                e["end"].as_str().unwrap().to_string(),
                e["busy"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[test]
fn dam_agenda_json_answers_every_event_in_the_window_as_intervals() {
    let _guard = support::guard("dam_agenda_json_answers_every_event_in_the_window_as_intervals");
    let sb = Sandbox::new();
    sb.new_object(&[
        "--event",
        "standup",
        "--start",
        "2026-09-25T14:00",
        "--end",
        "2026-09-25T14:30",
    ]);
    sb.new_object(&[
        "--event",
        "offsite",
        "--start",
        "2026-09-25",
        "--end",
        "2026-09-26",
    ]);
    sb.new_object(&[
        "--event",
        "next week",
        "--start",
        "2026-10-02T09:00",
        "--end",
        "2026-10-02T10:00",
    ]);
    let (ok, out, err) = sb.dam(&[
        "agenda",
        "--from",
        "2026-09-25",
        "--to",
        "2026-09-26",
        "--json",
    ]);
    assert!(ok, "{err}");
    assert_eq!(
        busy_intervals(&out),
        vec![
            ("2026-09-25".into(), "2026-09-26".into(), true),
            (
                "2026-09-25T14:00:00+00:00".into(),
                "2026-09-25T14:30:00+00:00".into(),
                true
            )
        ]
    );
}

#[test]
fn max_age_refuses_a_remote_never_pulled_and_answers_once_one_lands() {
    let _guard = support::guard("max_age_refuses_a_remote_never_pulled_and_answers_once_one_lands");
    let sb = Sandbox::new();
    let never = sb.output(&["agenda", "--json", "--max-age", "fake=1h"]);
    assert_eq!(never.status.code(), Some(4));
    assert!(never.stdout.is_empty(), "a refused read prints no document");
    let refusal = error_document(&never);
    assert_eq!(refusal["error"]["rule"], "stale_remote");
    assert_eq!(
        refusal["error"]["message"],
        "remote \"fake\" has never been pulled; run dam pull fake"
    );

    let (ok, _, err) = sb.dam(&["pull", "fake"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["agenda", "--json", "--max-age", "fake=1h"]);
    assert!(ok, "{err}");
    assert_eq!(busy_intervals(&out), vec![]);
}

#[test]
fn max_age_takes_a_configured_remote_and_a_duration_it_can_read() {
    let _guard = support::guard("max_age_takes_a_configured_remote_and_a_duration_it_can_read");
    let sb = Sandbox::new();
    let unknown = sb.output(&["agenda", "--json", "--max-age", "nope=1h"]);
    assert_eq!(unknown.status.code(), Some(4));
    assert_eq!(error_document(&unknown)["error"]["rule"], "no_such_remote");
    for unreadable in ["fake", "fake=", "=1h", "fake=soon", "fake=1d"] {
        let out = sb.output(&["agenda", "--max-age", unreadable]);
        assert_eq!(out.status.code(), Some(2), "{unreadable}");
    }
}
