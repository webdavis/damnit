use super::*;

fn read(text: &str) -> MutationResult {
    serde_json::from_str(text).unwrap()
}

#[test]
fn an_accepted_result_carrying_a_reason_reads_as_accepted_without_it() {
    assert_eq!(
        read(r#"{"oid":"o1","ok":true,"remote_id":"r1","why":"ignored"}"#).verdict,
        Verdict::Accepted {
            remote_id: Some("r1".into())
        }
    );
}

#[test]
fn a_refusal_without_a_reason_says_the_remote_gave_none() {
    assert_eq!(
        read(r#"{"oid":"o1","ok":false}"#).verdict,
        Verdict::Refused {
            why: "the remote gave no reason".into()
        }
    );
}

#[test]
fn a_refusal_is_written_as_ok_false_with_its_reason() {
    let refused = MutationResult {
        oid: "o1".into(),
        verdict: Verdict::Refused {
            why: "read-only".into(),
        },
    };
    assert_eq!(
        serde_json::to_string(&refused).unwrap(),
        r#"{"oid":"o1","ok":false,"why":"read-only"}"#
    );
}

#[test]
fn a_result_that_is_not_a_json_object_is_refused() {
    for text in [r#"{"results":[["o1",true]]}"#, r#"{"results":["o1"]}"#] {
        let refused = serde_json::from_str::<crate::PushResponse>(text).unwrap_err();
        assert!(
            refused.to_string().starts_with("expected a JSON object"),
            "{text}: {refused}"
        );
    }
}
