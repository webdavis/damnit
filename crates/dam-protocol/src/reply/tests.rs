use super::*;
use crate::{Capabilities, PullResponse, PushResponse};

#[test]
fn an_error_response_deserializes() {
    let r: Reply<PullResponse> = serde_json::from_str(r#"{"error":"no such account"}"#).unwrap();
    assert_eq!(
        r,
        Reply::Failed {
            error: "no such account".into()
        }
    );
}

#[test]
fn a_response_carrying_error_and_a_shapes_own_keys_reads_as_the_error() {
    let read: Reply<PushResponse> =
        serde_json::from_str(r#"{"error":"quota","results":[]}"#).unwrap();
    assert_eq!(
        read,
        Reply::Failed {
            error: "quota".into()
        }
    );
}

#[test]
fn an_answer_is_read_as_the_type_asked_for_whatever_other_keys_it_carries() {
    let read: Reply<PullResponse> = serde_json::from_str(r#"{"objects":[],"results":[]}"#).unwrap();
    assert_eq!(read, Reply::Answered(PullResponse::default()));
}

fn refusal<T: DeserializeOwned + std::fmt::Debug>(text: &str) -> String {
    serde_json::from_str::<Reply<T>>(text)
        .unwrap_err()
        .to_string()
}

#[test]
fn an_answer_that_is_not_a_json_object_is_refused_for_every_request() {
    for text in [r#"[[]]"#, r#"[1,["task"],["subject"],[],false]"#, r#""ok""#] {
        let refused = [
            refusal::<Capabilities>(text),
            refusal::<PullResponse>(text),
            refusal::<PushResponse>(text),
        ];
        assert_eq!(
            refused,
            ["expected a JSON object"; 3].map(String::from),
            "{text}"
        );
    }
}
