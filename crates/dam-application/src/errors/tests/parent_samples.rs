use crate::Refusal;
use crate::testing::oid;

pub(super) fn parent_samples() -> Vec<Refusal> {
    vec![Refusal::ParentNotKnown {
        oid: oid(1),
        unsettled: vec![oid(3)],
        namesakes: vec![oid(2)],
        folder: dam_domain::Path::parse("Home/errands").unwrap(),
        held_by_a_remote: false,
    }]
}
