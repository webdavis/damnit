use crate::testing::oid;
use crate::{Refusal, Retry};
use dam_domain::Path;

pub(super) fn parent_samples() -> Vec<Refusal> {
    vec![
        Refusal::ParentNotKnown {
            oid: oid(1),
            unsettled: vec![oid(3)],
            namesakes: vec![oid(2)],
            folder: Path::parse("Home/errands").unwrap(),
            held_by_a_remote: false,
        },
        Refusal::PathOutsideParent {
            parent: oid(2),
            folder: Path::parse("Home/errands").unwrap(),
            path: Path::parse("Work").unwrap(),
        },
        Refusal::SeveralHoldThePath {
            namesakes: vec![oid(1), oid(2)],
            folder: Path::parse("Home/errands").unwrap(),
            retry: Retry::New("buy milk".into()),
        },
    ]
}
