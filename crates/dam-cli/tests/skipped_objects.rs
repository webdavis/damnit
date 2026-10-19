mod support;

use support::sandbox::Sandbox;

const LOOPING_UP_7: &str = r#"{"objects":[],"rejected":[{"remote_id":"up-7","why":"its parent chain loops"}],"removed":[],"sync":null}"#;

#[test]
fn an_object_a_pull_skipped_reads_as_skipped_not_as_a_failed_pull() {
    let _guard = support::guard("an_object_a_pull_skipped_reads_as_skipped_not_as_a_failed_pull");
    let sb = Sandbox::new();
    sb.install_helper_answering_pull(LOOPING_UP_7);
    let (ok, _, err) = sb.dam(&["pull"]);
    assert!(ok, "{err}");

    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("pull from fake skipped up-7: its parent chain loops"),
        "{out}"
    );
    assert!(!out.contains("failed"), "{out}");

    let (ok, out, err) = sb.dam(&["status", "--json"]);
    assert!(ok, "{err}");
    let status: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        status["notices"],
        serde_json::json!([{
            "kind": "pull_skipped",
            "remote": "fake",
            "remote_id": "up-7",
            "why": "its parent chain loops"
        }])
    );
}

const CLEAN_UP_7: &str = r#"{"objects":[{"oid":"","remote_id":"up-7","kind":"task","subject":"from upstream","body":"","path":"","labels":[],"depends":[],"reminders":[],"recurrence":null,"task":{"done":false,"priority":2,"due":null,"deadline":null,"event":null},"event":null}],"removed":[],"sync":null}"#;

fn skipped_lines(sb: &Sandbox) -> usize {
    let (ok, out, err) = sb.dam(&["status"]);
    assert!(ok, "{err}");
    out.lines().filter(|l| l.contains("skipped up-7")).count()
}

#[test]
fn the_same_object_skipped_on_every_pull_is_noticed_once_until_it_reads_cleanly() {
    let _guard = support::guard(
        "the_same_object_skipped_on_every_pull_is_noticed_once_until_it_reads_cleanly",
    );
    let sb = Sandbox::new();
    sb.install_helper_answering_pull(LOOPING_UP_7);
    assert!(sb.dam(&["pull"]).0);
    assert!(sb.dam(&["pull"]).0);
    assert_eq!(skipped_lines(&sb), 1);

    sb.install_helper_answering_pull(CLEAN_UP_7);
    assert!(sb.dam(&["pull"]).0);
    assert_eq!(skipped_lines(&sb), 0);

    sb.install_helper_answering_pull(LOOPING_UP_7);
    assert!(sb.dam(&["pull"]).0);
    assert_eq!(skipped_lines(&sb), 1);
}
