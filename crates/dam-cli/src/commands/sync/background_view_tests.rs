use std::time::Duration;

use dam_application::{BackgroundReport, BackgroundStep, Change};

use super::background_view::BackgroundSynced;
use crate::view;

fn shown(done: Vec<(BackgroundStep, Change)>, minutes: u64) -> String {
    let report = BackgroundReport {
        remote: "todoist".into(),
        interval: Duration::from_secs(minutes * 60),
        done,
    };
    view::report(&BackgroundSynced::new(&report)).unwrap().human
}

#[test]
fn each_step_says_what_is_now_true() {
    use BackgroundStep::*;
    use Change::*;
    let cases = [
        (
            Start,
            Now,
            "todoist: syncing in the background every 15m until the next restart",
        ),
        (
            Start,
            Already,
            "todoist: already syncing in the background every 15m until the next restart",
        ),
        (Stop, Now, "todoist: background sync stopped"),
        (Stop, Already, "todoist: background sync was not running"),
        (
            Enable,
            Now,
            "todoist: background sync starts at every login, every 15m",
        ),
        (
            Enable,
            Already,
            "todoist: background sync already starts at every login, every 15m",
        ),
        (
            Disable,
            Now,
            "todoist: background sync no longer starts at login",
        ),
        (
            Disable,
            Already,
            "todoist: background sync already does not start at login",
        ),
    ];
    for (step, change, line) in cases {
        assert_eq!(shown(vec![(step, change)], 15), line);
    }
}

#[test]
fn enable_now_says_both_and_an_hour_reads_as_hours() {
    let done = vec![
        (BackgroundStep::Enable, Change::Now),
        (BackgroundStep::Start, Change::Now),
    ];
    assert_eq!(
        shown(done, 60),
        "todoist: background sync starts at every login, every 1h\n\
         todoist: syncing in the background every 1h until the next restart"
    );
}

#[test]
fn the_json_names_each_step_and_whether_it_changed_anything() {
    let report = BackgroundReport {
        remote: "todoist".into(),
        interval: Duration::from_secs(300),
        done: vec![
            (BackgroundStep::Disable, Change::Already),
            (BackgroundStep::Stop, Change::Now),
        ],
    };
    let json = view::report(&BackgroundSynced::new(&report)).unwrap().data;
    for verb in ["start", "stop", "enable", "disable"] {
        let path = format!(
            "{}/../../docs/schemas/sync-{verb}.schema.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let schema = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert!(jsonschema::is_valid(&schema, &json), "{verb}: {json}");
    }
    assert_eq!(
        json,
        serde_json::json!({
            "remote": "todoist",
            "interval_seconds": 300,
            "steps": [
                {"step": "disable", "changed": false},
                {"step": "stop", "changed": true}
            ]
        })
    );
}
