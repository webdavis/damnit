mod support;

use serde_json::{Value, json};
use support::errors::error_document;
use support::request::{request_file, without_oids_hashes_or_times};
use support::sandbox::Sandbox;
use support::token_table::TokenTable;
use support::tokens::{answer, grant, token};

fn requiring_a_session(sb: &Sandbox) -> String {
    let held = token(1).expose();
    sb.add_token(
        "claude",
        TokenTable::valued(&held)
            .require(&["session"])
            .grant(grant(&["task:create"])),
    );
    held
}

fn sent(out: &std::process::Output) -> Value {
    without_oids_hashes_or_times(answer(out, "sent.schema.json"))
}

#[test]
fn a_tokens_request_with_meta_is_sent_as_its_flags_are() {
    let _guard = support::guard("a_tokens_request_with_meta_is_sent_as_its_flags_are");
    let by_flags = Sandbox::new();
    let held = requiring_a_session(&by_flags);
    let line = [
        "new",
        "bread",
        "--path",
        "inbox",
        "--meta",
        "session=s1",
        "--json",
    ];
    let flags_answer = sent(&by_flags.output_holding(&held, &line));

    let by_file = Sandbox::new();
    let held = requiring_a_session(&by_file);
    let path = request_file(
        &by_file,
        &json!({"subject": "bread", "path": "inbox", "meta": {"session": "s1"}}).to_string(),
    );
    let file_answer = sent(&by_file.output_holding(&held, &["new", "--json", "--request", &path]));
    assert_eq!(file_answer, flags_answer);
}

#[test]
fn a_tokens_request_without_the_required_meta_is_refused_as_its_flags_are() {
    let _guard =
        support::guard("a_tokens_request_without_the_required_meta_is_refused_as_its_flags_are");
    let sb = Sandbox::new();
    let held = requiring_a_session(&sb);
    let path = request_file(&sb, r#"{"subject": "bread"}"#);
    let out = sb.output_holding(&held, &["new", "--json", "--request", &path]);
    assert_eq!(out.status.code(), Some(5), "{out:?}");
    assert_eq!(error_document(&out)["error"]["rule"], "missing_details");
}
