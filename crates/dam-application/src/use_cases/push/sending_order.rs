use std::cmp::Reverse;

use dam_domain::Change;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Turn {
    PlacedOutermostFirst(usize),
    RemovedInnermostFirst(Reverse<usize>),
}

pub(super) fn turn(change: &Change) -> Turn {
    let depth = change.object().base().path.depth();
    match change {
        Change::Delete { .. } => Turn::RemovedInnermostFirst(Reverse(depth)),
        _ => Turn::PlacedOutermostFirst(depth),
    }
}
