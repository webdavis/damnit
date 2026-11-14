use super::*;
use dam_domain::Oid;

#[test]
fn an_unanswered_push_result_reaches_the_application_with_its_reason() {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(3));
    let outcomes = outcomes_from_wire(vec![MutationResult {
        oid: oid.to_string(),
        verdict: Verdict::Unanswered {
            why: "Todoist did not answer in time".into(),
        },
    }]);
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
