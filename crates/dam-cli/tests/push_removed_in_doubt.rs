mod support;

use serde_json::{Value, json};
use support::config_files::Remote;
use support::sandbox::Sandbox;

fn removed_in_doubt_notices(sb: &Sandbox) -> Vec<Value> {
    let status = support::status_json(sb, &["--json"]);
    support::schema::assert_matches_schema(&status, "status.schema.json");
    status["notices"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "create_in_doubt_removed")
        .cloned()
        .collect()
}

fn the_create_left_unanswered(sb: &Sandbox) -> (String, String) {
    let status = support::status_json(sb, &["--json"]);
    let oid = status["unpushed"][0]["oids"][0]
        .as_str()
        .unwrap()
        .to_string();
    let (ok, out, err) = sb.dam(&["ls", "--json", "--no-pull"]);
    assert!(ok, "{err}");
    let listed: Value = serde_json::from_str(&out).unwrap();
    let object = listed["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["oid"] == oid.as_str())
        .unwrap();
    (oid, object["subject"].as_str().unwrap().to_string())
}

#[test]
fn a_create_in_doubt_removed_before_its_resend_is_noticed_once() {
    let _guard = support::guard("a_create_in_doubt_removed_before_its_resend_is_noticed_once")
        .allow_slow("waits out a two second deadline on a push that hangs");
    let sb = Sandbox::new();
    sb.add_remote("fake", Remote::fake().deadline("2s"));
    for subject in ["call Ada", "call tool"] {
        sb.new_object(&[subject, "--path", "work/"]);
        assert!(sb.dam(&["add", "-A"]).0);
        assert!(sb.dam(&["commit", "-m", subject]).0);
    }
    sb.hang_the_next_push(true);
    assert!(!sb.dam(&["push"]).0);
    let (oid, subject) = the_create_left_unanswered(&sb);
    assert!(sb.dam(&["rm", &oid]).0);
    assert!(sb.dam(&["add", "-A"]).0);
    assert!(sb.dam(&["commit", "-m", "drop it"]).0);

    let (ok, _, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    let notice = format!(
        "the create of {subject:?} in work/ got no answer from fake, and it was then removed \
         here; fake may hold a copy to remove by hand"
    );
    assert!(out.contains(&notice), "{out}");
    assert_eq!(
        removed_in_doubt_notices(&sb),
        vec![
            json!({"kind": "create_in_doubt_removed", "remote": "fake", "oid": oid, "subject": subject, "path": "work/"})
        ]
    );

    let (ok, _, err) = sb.dam(&["push"]);
    assert!(ok, "{err}");
    assert_eq!(removed_in_doubt_notices(&sb).len(), 1, "left once");
}
