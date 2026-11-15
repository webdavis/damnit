use super::*;
use dam_domain::Oid;

#[test]
fn an_unanswered_push_result_reaches_the_application_with_its_reason() {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(3));
    let outcomes = push_from_wire(PushResponse {
        results: vec![dam_protocol::MutationResult {
            oid: oid.to_string(),
            verdict: Verdict::Unanswered {
                why: "Todoist did not answer in time".into(),
            },
        }],
        stopped: None,
    })
    .outcomes;
    assert_eq!(
        outcomes,
        vec![MutationOutcome {
            oid,
            verdict: MutationVerdict::Unanswered {
                why: "Todoist did not answer in time".into()
            },
        }]
    );
}

#[test]
fn a_refused_push_result_reaches_the_application_with_its_reason_and_whether_it_was_asked() {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(3));
    let outcomes = push_from_wire(PushResponse {
        results: vec![dam_protocol::MutationResult {
            oid: oid.to_string(),
            verdict: Verdict::Refused {
                why: "no such project".into(),
                unasked: true,
            },
        }],
        stopped: None,
    })
    .outcomes;
    assert_eq!(
        outcomes,
        vec![MutationOutcome {
            oid,
            verdict: MutationVerdict::Refused {
                why: "no such project".into(),
                unasked: true,
            },
        }]
    );
}
