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

#[test]
fn an_unanswered_result_round_trips_with_its_reason() {
    let unanswered = MutationResult {
        oid: "o1".into(),
        verdict: Verdict::Unanswered {
            why: "the remote did not answer in time".into(),
        },
    };
    let text = serde_json::to_string(&unanswered).unwrap();
    assert_eq!(read(&text), unanswered);
    assert!(!unanswered.ok());
    assert_eq!(unanswered.why(), Some("the remote did not answer in time"));
}

#[test]
fn an_unanswered_result_has_no_ok_so_a_reader_that_needs_one_cannot_take_it_for_a_refusal() {
    let unanswered = MutationResult {
        oid: "o1".into(),
        verdict: Verdict::Unanswered { why: "lost".into() },
    };
    assert_eq!(
        serde_json::to_string(&unanswered).unwrap(),
        r#"{"oid":"o1","unanswered":true,"why":"lost"}"#
    );
}

#[test]
fn a_result_with_neither_ok_nor_unanswered_is_refused() {
    let refused = serde_json::from_str::<MutationResult>(r#"{"oid":"o1"}"#).unwrap_err();
    assert!(refused.to_string().contains("ok"), "{refused}");
}

#[test]
fn a_result_list_that_leaves_a_mutation_out_still_parses() {
    let answer: crate::PushResponse = serde_json::from_str(
        r#"{"results":[{"oid":"o1","ok":true},{"oid":"o3","unanswered":true,"why":"lost"}]}"#,
    )
    .unwrap();
    let oids: Vec<&str> = answer.results.iter().map(|r| r.oid.as_str()).collect();
    assert_eq!(oids, vec!["o1", "o3"]);
}

#[test]
fn push_with_an_unanswered_result_round_trips_the_fixture() {
    let text = include_str!("../../../fixtures/push-unanswered.response.json").trim_end();
    let answer: crate::PushResponse = serde_json::from_str(text).unwrap();
    assert_eq!(
        answer.results[1].verdict,
        Verdict::Unanswered {
            why: "Todoist did not answer in time".into()
        }
    );
    assert_eq!(serde_json::to_string(&answer).unwrap(), text);
}
