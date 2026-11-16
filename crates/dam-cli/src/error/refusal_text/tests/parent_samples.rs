use dam_application::Refusal;
use dam_domain::Path;

use super::oid;

pub(super) fn parent_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    vec![
        (
            Refusal::ParentNotKnown {
                oid: oid(1),
                unsettled: vec![oid(3)],
                namesakes: vec![oid(2)],
                folder: Path::parse("Home/errands").unwrap(),
                held_by_a_remote: true,
            },
            "dam does not know whether 0101010 or 0202020 holds 0303030, so it cannot tell what moves with 0101010; run: dam pull, or name it: dam mv 0303030 --parent 0101010",
        ),
        (
            Refusal::ParentNotKnown {
                oid: oid(1),
                unsettled: vec![oid(3), oid(4), oid(5)],
                namesakes: vec![oid(2), oid(6)],
                folder: Path::parse("Home/errands").unwrap(),
                held_by_a_remote: false,
            },
            "dam does not know whether 0101010, 0202020 or 0606060 holds 0303030 and 2 more objects in Home/errands/, so it cannot tell what moves with 0101010; name it: dam mv 0303030 --parent 0101010",
        ),
    ]
}
