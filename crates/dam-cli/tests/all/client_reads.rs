use crate::support;

use support::sandbox::Sandbox;

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
        let (ok, out, err) = sb.dam(&[
            "edit",
            &oid,
            "--due",
            word,
            "--deadline",
            word,
            "-o",
            "json",
        ]);
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
    let (ok, _, err) = sb.dam(&["show", &oid, "-o", "json"]);
    assert!(ok, "{err}");

    for flag in ["--due", "--deadline", "--start", "--end"] {
        let out = sb.output(&["edit", &oid, flag, "someday", "-o", "json"]);
        assert_eq!(out.status.code(), Some(2), "{flag} read a word it cannot");
        let said = String::from_utf8_lossy(&out.stderr);
        let document: serde_json::Value = serde_json::from_str(&said).unwrap();
        assert_eq!(document["error"]["kind"], "usage", "{said}");
        let message = document["error"]["message"].as_str().unwrap();
        for form in [
            "today",
            "today+3",
            "now+2h",
            "next mon",
            "in 3 days",
            "2026-11-20",
        ] {
            assert!(message.contains(form), "{flag}: {message}");
        }
    }
}
