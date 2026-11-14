use dam_domain::{Blockers, ItemHash};

use crate::ItemRefusal;
use crate::testing::oid;

fn hash() -> ItemHash {
    ItemHash::new(oid(1))
}

pub(super) fn item_samples() -> Vec<ItemRefusal> {
    vec![
        ItemRefusal::NothingChanged,
        ItemRefusal::Declined(hash()),
        ItemRefusal::NotAnObject(hash()),
        ItemRefusal::AlreadyCommitted(hash()),
        ItemRefusal::NotYours(hash()),
        ItemRefusal::NotWaiting {
            hash: hash(),
            outcome: "added",
        },
        ItemRefusal::TargetGone {
            hash: hash(),
            target: oid(2),
        },
        ItemRefusal::TargetNotCommitted {
            hash: hash(),
            target: oid(2),
        },
        ItemRefusal::TargetStaged {
            hash: hash(),
            target: oid(2),
        },
        ItemRefusal::Clash {
            hash: hash(),
            target: oid(2),
            fields: vec![dam_domain::Field::Subject],
        },
        ItemRefusal::RemovesAChangedTask {
            hash: hash(),
            target: oid(2),
            fields: vec![dam_domain::Field::Subject],
        },
        ItemRefusal::Cycle {
            hash: hash(),
            target: oid(2),
            path: vec![oid(2)],
        },
        ItemRefusal::Blocked {
            hash: hash(),
            target: oid(2),
            blockers: Blockers::default(),
        },
        ItemRefusal::EditedSinceAdded {
            hash: hash(),
            target: oid(2),
        },
        ItemRefusal::SubjectNamesNoFolder {
            hash: hash(),
            target: oid(2),
        },
    ]
}
