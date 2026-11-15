use crate::support;

use support::sandbox::Sandbox;

#[test]
fn remote_list_reports_when_each_remote_was_last_pulled_and_pushed() {
    let _guard = support::guard("remote_list_reports_when_each_remote_was_last_pulled_and_pushed");
    let sb = Sandbox::new();

    let fresh = the_one_sandbox_remote_as_listed(&sb);
    assert_eq!(fresh["last_pull"], serde_json::Value::Null, "{fresh}");
    assert_eq!(fresh["last_push"], serde_json::Value::Null, "{fresh}");

    let (ok, _, err) = sb.dam(&["pull", "--json"]);
    assert!(ok, "{err}");
    let pulled = the_one_sandbox_remote_as_listed(&sb);
    assert!(pulled["last_pull"].as_str().is_some(), "{pulled}");
    assert_eq!(pulled["last_push"], serde_json::Value::Null, "{pulled}");

    sb.new_object(&["buy oat milk"]);
    let (ok, _, err) = sb.dam(&["add", "-A"]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["commit", "-m", "triage"]);
    assert!(ok, "{err}");
    let (ok, _, err) = sb.dam(&["push", "--json"]);
    assert!(ok, "{err}");
    let pushed = the_one_sandbox_remote_as_listed(&sb);
    let at = pushed["last_push"].as_str().expect("a push time");
    at.parse::<jiff::Timestamp>()
        .unwrap_or_else(|e| panic!("last_push is not RFC 3339: {e} ({at})"));
    assert_eq!(pushed["last_pull"], pulled["last_pull"], "{pushed}");

    let (ok, human, err) = sb.dam(&["remote", "list"]);
    assert!(ok, "{err}");
    assert!(human.contains("pulled "), "{human}");
    assert!(human.contains("pushed "), "{human}");
}

fn the_one_sandbox_remote_as_listed(sb: &Sandbox) -> serde_json::Value {
    let (ok, out, err) = sb.dam(&["remote", "list", "--json"]);
    assert!(ok, "{err}");
    let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
    answer["remotes"][0].clone()
}

#[test]
fn every_date_flag_accepts_the_words_and_refuses_anything_else() {
    let _guard = support::guard("every_date_flag_accepts_the_words_and_refuses_anything_else");
    let sb = Sandbox::new();

    for word in [
        "today",
        "tomorrow",
        "mon",
        "MONDAY",
        "next fri",
        "in 3 days",
        "in 2 months",
    ] {
        let oid = sb.new_object(&["a task", "--due", word, "--deadline", word]);
        let (ok, out, err) = sb.dam(&["edit", &oid, "--due", word, "--deadline", word, "--json"]);
        assert!(ok, "{word}: {err}");
        let answer: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(answer["task"]["due"].is_string(), "{word}: {answer}");
        assert!(answer["task"]["deadline"].is_string(), "{word}: {answer}");
    }
    let oid = sb.new_object(&[
        "an event",
        "--event",
        "--start",
        "mon",
        "--end",
        "in 1 week",
    ]);
    let (ok, _, err) = sb.dam(&["show", &oid, "--json"]);
    assert!(ok, "{err}");

    for flag in ["--due", "--deadline", "--start", "--end"] {
        let out = sb.output(&["edit", &oid, flag, "someday", "--json"]);
        assert_eq!(out.status.code(), Some(2), "{flag} read a word it cannot");
        let said = String::from_utf8_lossy(&out.stderr);
        let document: serde_json::Value = serde_json::from_str(&said).unwrap();
        assert_eq!(document["error"]["kind"], "usage", "{said}");
        let message = document["error"]["message"].as_str().unwrap();
        for form in ["today", "next <weekday>", "in <n> days", "YYYY-MM-DD"] {
            assert!(message.contains(form), "{flag}: {message}");
        }
    }
}
