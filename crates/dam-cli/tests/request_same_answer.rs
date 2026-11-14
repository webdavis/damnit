mod support;

use serde_json::{Value, json};
use support::request::{answered, by_request, fed, without_oids_hashes_or_times};
use support::sandbox::Sandbox;

fn stored(sb: &Sandbox) -> Value {
    let (ok, out, err) = sb.dam(&["status", "--json", "--full"]);
    assert!(ok, "{err}");
    let mut status: Value = serde_json::from_str(&out).unwrap();
    for section in ["staged", "unstaged"] {
        if let Some(rows) = status[section].as_array_mut() {
            rows.sort_by_key(|row| row["subject"].as_str().unwrap_or_default().to_string());
        }
    }
    without_oids_hashes_or_times(status)
}

fn a_task(sb: &Sandbox) -> String {
    sb.new_object(&["milk", "--due", "2026-12-01", "--labels", "errand"])
}

fn same_as_flags(flags: &[&str], command: &[&str], request: impl Fn(&str) -> Value) {
    let by_flags = Sandbox::new();
    let oid = a_task(&by_flags);
    let line: Vec<&str> = flags
        .iter()
        .map(|w| if *w == "OID" { &oid } else { *w })
        .collect();
    let flags_answer = answered(&by_flags.output(&[&line[..], &["--json"]].concat()));

    let by_file = Sandbox::new();
    let oid = a_task(&by_file);
    let file_answer = answered(&by_request(&by_file, command, &request(&oid)));

    assert_eq!(file_answer, flags_answer);
    assert_eq!(stored(&by_file), stored(&by_flags));
}

#[test]
fn a_new_request_answers_and_stores_what_its_flags_do() {
    let _guard = support::guard("a_new_request_answers_and_stores_what_its_flags_do");
    same_as_flags(
        &[
            "new",
            "bread",
            "--due",
            "2026-12-02",
            "--labels",
            "errand",
            "--labels",
            "home",
            "-p",
            "2",
            "--path",
            "home/",
            "--body=-sourdough",
            "--meta",
            "session=s1",
        ],
        &["new"],
        |_| {
            json!({"subject": "bread", "due": "2026-12-02", "labels": ["errand", "home"],
                   "priority": 2, "path": "home/", "body": "-sourdough",
                   "meta": {"session": "s1"}})
        },
    );
}

#[test]
fn an_edit_request_answers_and_stores_what_its_flags_do() {
    let _guard = support::guard("an_edit_request_answers_and_stores_what_its_flags_do");
    same_as_flags(
        &[
            "edit",
            "OID",
            "--subject",
            "oat milk",
            "--no-due",
            "--estimate",
            "30",
            "--unlabels",
            "errand",
        ],
        &["edit"],
        |oid| json!({"oid": oid, "subject": "oat milk", "no_due": true, "estimate": 30, "unlabels": ["errand"]}),
    );
}

#[test]
fn done_mv_and_rm_requests_answer_and_store_what_their_flags_do() {
    let _guard = support::guard("done_mv_and_rm_requests_answer_and_store_what_their_flags_do");
    same_as_flags(&["done", "OID"], &["done"], |oid| json!({"oid": oid}));
    same_as_flags(
        &["mv", "OID", "work/"],
        &["mv"],
        |oid| json!({"oid": oid, "path": "work/"}),
    );
    same_as_flags(&["rm", "OID"], &["rm"], |oid| json!({"oid": oid}));
}

fn tokens(sb: &Sandbox) -> Value {
    let (ok, out, err) = sb.dam(&["token", "list", "--json"]);
    assert!(ok, "{err}");
    let mut listed: Value = serde_json::from_str(&out).unwrap();
    listed["tokens"][0]["key_id"] = Value::Null;
    without_oids_hashes_or_times(listed)
}

#[test]
fn a_token_create_request_makes_the_token_its_flags_do() {
    let _guard = support::guard("a_token_create_request_makes_the_token_its_flags_do");
    let by_flags = Sandbox::new();
    let mut flags_answer = answered(&by_flags.output(&[
        "token",
        "create",
        "tool",
        "--allow",
        "task:read",
        "--paths",
        "work",
        "--expires",
        "2999-05-11T00:00:00Z",
        "--require",
        "session",
        "--json",
    ]));
    let by_file = Sandbox::new();
    let request = json!({"name": "tool", "allow": ["task:read"], "paths": ["work"],
                         "expires": "2999-05-11T00:00:00Z", "require": ["session"]});
    let mut file_answer = answered(&by_request(&by_file, &["token", "create"], &request));
    for answer in [&mut flags_answer, &mut file_answer] {
        answer["value"] = Value::Null;
        answer["key_id"] = Value::Null;
    }
    assert_eq!(file_answer, flags_answer);
    assert_eq!(tokens(&by_file), tokens(&by_flags));
}

#[test]
fn a_request_on_standard_input_and_in_toon_is_read_the_same() {
    let _guard = support::guard("a_request_on_standard_input_and_in_toon_is_read_the_same");
    let by_flags = Sandbox::new();
    let flags_answer = answered(&by_flags.output(&["new", "bread", "--labels", "home", "--json"]));
    for document in [
        json!({"subject": "bread", "labels": ["home"]}).to_string(),
        "subject: bread\nlabels[1]: home\n".to_string(),
    ] {
        let by_stdin = Sandbox::new();
        let out = fed(&by_stdin, &["new", "--json", "--request", "-"], &document);
        assert_eq!(answered(&out), flags_answer, "{document}");
        assert_eq!(stored(&by_stdin), stored(&by_flags));
    }
}

#[test]
fn day_set_reads_its_document_from_a_request_file_or_from_standard_input() {
    let _guard =
        support::guard("day_set_reads_its_document_from_a_request_file_or_from_standard_input");
    let day = json!({"day": "2026-11-11", "decisions": ["rest"]});
    let by_stdin = Sandbox::new();
    let stdin_answer = answered(&fed(&by_stdin, &["day", "set", "--json"], &day.to_string()));
    let by_file = Sandbox::new();
    assert_eq!(
        answered(&by_request(&by_file, &["day", "set"], &day)),
        stdin_answer
    );
    let by_dash = Sandbox::new();
    let dash = fed(
        &by_dash,
        &["day", "set", "--json", "--request", "-"],
        &day.to_string(),
    );
    assert_eq!(answered(&dash), stdin_answer);
}
