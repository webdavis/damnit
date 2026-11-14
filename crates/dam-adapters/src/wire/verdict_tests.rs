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
