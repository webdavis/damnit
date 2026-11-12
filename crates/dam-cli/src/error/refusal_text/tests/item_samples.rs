use dam_application::{ItemRefusal, Refusal};
use dam_domain::{Blockers, Field, ItemHash};

use super::oid;

fn hash() -> ItemHash {
    ItemHash::new(oid(1))
}

pub(super) fn item_refusals_with_their_text() -> Vec<(Refusal, &'static str)> {
    [
        (
            ItemRefusal::NothingChanged,
            "that leaves the object as it is",
        ),
        (
            ItemRefusal::Declined(hash()),
            "the owner declined this as 0101010; it is not offered again",
        ),
        (
            ItemRefusal::NotAnObject(hash()),
            "0101010 is a waiting item, not an object; dam add or dam rm it",
        ),
        (
            ItemRefusal::NotYours(hash()),
            "0101010 was sent by another token; a token may remove only its own items",
        ),
        (
            ItemRefusal::NotWaiting {
                hash: hash(),
                outcome: "declined",
            },
            "0101010 was already declined",
        ),
        (
            ItemRefusal::TargetGone {
                hash: hash(),
                target: oid(2),
            },
            "0101010 changes 0202020, which is no longer in the working layer",
        ),
        (
            ItemRefusal::TargetNotCommitted {
                hash: hash(),
                target: oid(2),
            },
            "0101010 changes 0202020, which has never been committed; commit it, then add 0101010",
        ),
        (
            ItemRefusal::TargetStaged {
                hash: hash(),
                target: oid(2),
            },
            "0101010 changes 0202020, which already has a staged change; commit or reset it, then add 0101010",
        ),
        (
            ItemRefusal::Clash {
                hash: hash(),
                target: oid(2),
                fields: vec![Field::Subject, Field::Due],
            },
            "0101010 changes subject, due of 0202020, which you changed too; run dam resolve 0101010 --ours|--theirs",
        ),
        (
            ItemRefusal::Cycle {
                hash: hash(),
                target: oid(2),
                path: vec![oid(3), oid(2)],
            },
            "0101010 makes 0202020 depend on what leads back to it, a cycle through 0303030 -> 0202020; decline it with dam rm 0101010",
        ),
        (
            ItemRefusal::Blocked {
                hash: hash(),
                target: oid(2),
                blockers: Blockers {
                    dependencies: vec![oid(3)],
                    children: Vec::new(),
                },
            },
            "0101010 completes 0202020, which has 1 open dependency: 0303030; finish those first, or run dam done 0202020 --force and dam rm 0101010",
        ),
        (
            ItemRefusal::EditedSinceAdded {
                hash: hash(),
                target: oid(2),
            },
            "0202020 is new from 0101010, and you have changed it since you added it; commit it first",
        ),
        (
            ItemRefusal::SubjectNamesNoFolder {
                hash: hash(),
                target: oid(2),
            },
            "0101010 renames 0202020, whose folder holds objects, to a subject that cannot name a folder; decline it with dam rm 0101010",
        ),
    ]
    .into_iter()
    .map(|(refusal, text)| (Refusal::Item(refusal), text))
    .collect()
}
