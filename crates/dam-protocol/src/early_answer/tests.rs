use super::*;
use crate::{MutationResult, PushResponse, Reply, Request, Verdict};

const EARLY: &str = r#"{"early_answer":{"oid":"0101010101010101010101010101010101010101","ok":true,"remote_id":"r1"}}"#;

fn accepted() -> MutationResult {
    MutationResult {
        oid: "01".repeat(20),
        verdict: Verdict::Accepted {
            remote_id: Some("r1".into()),
        },
    }
}

#[test]
fn a_push_request_that_does_not_ask_reads_as_answer_once() {
    let request: Request = serde_json::from_str(r#"{"cmd":"push","mutations":[]}"#).unwrap();
    assert_eq!(
        request,
        Request::Push {
            mutations: vec![],
            early_answers: false,
        }
    );
    assert_eq!(
        serde_json::to_string(&request).unwrap(),
        r#"{"cmd":"push","mutations":[]}"#
    );
}

#[test]
fn a_push_request_asking_for_early_answers_says_so_on_the_wire() {
    let request = Request::Push {
        mutations: vec![],
        early_answers: true,
    };
    let line = serde_json::to_string(&request).unwrap();
    assert_eq!(
        line,
        r#"{"cmd":"push","mutations":[],"early_answers":true}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&line).unwrap(), request);
}

#[test]
fn a_request_with_a_field_the_helper_does_not_know_still_reads() {
    let request: Request =
        serde_json::from_str(r#"{"cmd":"push","mutations":[],"later_field":1}"#).unwrap();
    assert!(matches!(request, Request::Push { .. }));
}

#[test]
fn an_early_answer_round_trips() {
    let early = EarlyAnswer {
        early_answer: accepted(),
    };
    assert_eq!(serde_json::to_string(&early).unwrap(), EARLY);
    assert_eq!(serde_json::from_str::<EarlyAnswer>(EARLY).unwrap(), early);
}

#[test]
fn an_early_answer_is_never_read_as_a_final_reply() {
    assert!(serde_json::from_str::<Reply<PushResponse>>(EARLY).is_err());
}

#[test]
fn a_final_reply_is_never_read_as_an_early_answer() {
    let fin = r#"{"results":[{"oid":"0101010101010101010101010101010101010101","ok":true}]}"#;
    assert!(serde_json::from_str::<EarlyAnswer>(fin).is_err());
    assert!(serde_json::from_str::<EarlyAnswer>(r#"{"error":"nope"}"#).is_err());
}
