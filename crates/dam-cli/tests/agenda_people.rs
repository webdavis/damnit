mod support;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serde_json::{Value, json};
use support::config_files::Remote;
use support::sandbox::Sandbox;
use support::schema::assert_matches_schema;
use support::tokens::{grant, grant_to};

const WINDOW: [&str; 6] = [
    "agenda",
    "--from",
    "2026-09-25",
    "--to",
    "2026-09-26",
    "--no-pull",
];

fn pulled_from_the_calendar() -> Sandbox {
    let sb = Sandbox::new();
    let helper = sb.dir.path().join("bin/dam-remote-cal");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake_calendar_helper.sh"),
        &helper,
    )
    .unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
    sb.add_remote("cal", Remote::at("cal::").api_token("tok-cal"));
    let (ok, _, err) = sb.dam(&["pull", "cal"]);
    assert!(ok, "{err}");
    sb
}

fn agenda(sb: &Sandbox, format: &str) -> String {
    let mut args = WINDOW.to_vec();
    args.push(format);
    let (ok, out, err) = sb.dam(&args);
    assert!(ok, "{err}");
    out
}

fn row<'a>(document: &'a Value, subject: &str) -> &'a Value {
    document["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["subject"] == subject)
        .unwrap_or_else(|| panic!("no {subject} in {document}"))
}

#[test]
fn an_agenda_row_names_the_people_place_and_link_of_its_meeting() {
    let _guard = support::guard("an_agenda_row_names_the_people_place_and_link_of_its_meeting");
    let sb = pulled_from_the_calendar();
    let document: Value = serde_json::from_str(&agenda(&sb, "--json")).unwrap();
    assert_matches_schema(&document, "agenda.schema.json");
    let meeting = row(&document, "design review");
    assert_eq!(
        meeting["attendees"],
        json!([
            {"email": "ada@example.com", "response": "accepted", "self": true},
            {"email": "grace@example.com", "response": "tentative"}
        ])
    );
    assert_eq!(
        meeting["organizer"],
        json!({"email": "grace@example.com", "name": "Grace"})
    );
    assert_eq!(meeting["location"], "Room 4");
    assert_eq!(
        meeting["conference"],
        json!({"provider": "meet", "url": "https://meet.example.com/abc"})
    );
    assert_eq!(meeting["start"], "2026-09-25T14:00:00+00:00");
    assert_eq!(meeting["busy"], true);
}

#[test]
fn an_agenda_row_without_people_has_an_empty_list_and_no_organizer() {
    let _guard = support::guard("an_agenda_row_without_people_has_an_empty_list_and_no_organizer");
    let sb = pulled_from_the_calendar();
    let document: Value = serde_json::from_str(&agenda(&sb, "--json")).unwrap();
    let alone = row(&document, "focus block").as_object().unwrap();
    assert_eq!(alone["attendees"], json!([]));
    for absent in ["organizer", "location", "conference"] {
        assert!(!alone.contains_key(absent), "{absent} in {alone:?}");
    }
}

#[test]
fn the_toon_agenda_carries_the_same_people_as_the_json_one() {
    let _guard = support::guard("the_toon_agenda_carries_the_same_people_as_the_json_one");
    let sb = pulled_from_the_calendar();
    let from_json: Value = serde_json::from_str(&agenda(&sb, "--json")).unwrap();
    let from_toon: Value = toon_format::decode_default(&agenda(&sb, "--toon")).unwrap();
    assert_eq!(from_toon, from_json);
    assert_eq!(
        row(&from_toon, "design review")["organizer"]["email"],
        "grace@example.com"
    );
}

#[test]
fn a_token_sees_no_meeting_or_attendee_outside_its_grants() {
    let _guard = support::guard("a_token_sees_no_meeting_or_attendee_outside_its_grants");
    let sb = pulled_from_the_calendar();
    let held = grant_to(
        &sb,
        "reader",
        3,
        vec![grant(&["task:read", "event:read", "working:read"]).paths(&["work"])],
    );
    let mut args = WINDOW.to_vec();
    args.push("--json");
    let out = sb.output_holding(&held, &args);
    assert!(out.status.success(), "{out:?}");
    let document: Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut subjects: Vec<&str> = document["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["subject"].as_str().unwrap())
        .collect();
    subjects.sort();
    assert_eq!(subjects, ["design review", "focus block"]);
}
