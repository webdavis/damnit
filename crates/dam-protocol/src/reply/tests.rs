use super::*;
use crate::{PullResponse, PushResponse};

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
