use super::*;
use crate::Reply;

const STOPPED: &str = r#"{"results":[{"oid":"0101010101010101010101010101010101010101","ok":true,"remote_id":"r1"}],"stopped":"todoist is rate limiting this account; retry after 42s"}"#;

#[test]
fn a_push_response_without_a_stop_reads_as_not_stopped() {
    let r: Reply<PushResponse> = serde_json::from_str(r#"{"results":[]}"#).unwrap();
    assert_eq!(
        r,
        Reply::Answered(PushResponse {
            results: vec![],
            stopped: None,
        })
    );
}

#[test]
fn a_push_response_that_stopped_keeps_its_results_and_the_reason() {
    let answer = Reply::Answered(PushResponse {
        results: vec![MutationResult {
            oid: "01".repeat(20),
            verdict: Verdict::Accepted {
                remote_id: Some("r1".into()),
            },
        }],
        stopped: Some("todoist is rate limiting this account; retry after 42s".into()),
    });
    assert_eq!(serde_json::to_string(&answer).unwrap(), STOPPED);
    assert_eq!(
        serde_json::from_str::<Reply<PushResponse>>(STOPPED).unwrap(),
        answer
    );
}
