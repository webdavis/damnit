use dam_domain::Oid;

use super::Holding;
use crate::use_cases::push::removals::Removals;

impl Removals {
    pub(super) fn why_held(&self, stuck: &Oid, holding: &Holding) -> String {
        let short = stuck.short();
        if let Holding::PlaceNotKnown = holding {
            return format!(
                "where {short} sits is not known upstream; run: dam pull, then dam push"
            );
        }
        if let Holding::MaybeHeld(could_hold) = holding {
            let shorts: Vec<&str> = could_hold.iter().map(Oid::short).collect();
            let (last, rest) = shorts.split_last().unwrap_or((&"", &[]));
            return format!(
                "{short} may be held by {} or {last} upstream; run: dam pull, then dam push",
                rest.join(", ")
            );
        }
        if self.tried.contains(stuck) {
            format!("waits on {short} leaving it upstream, which this push did not do")
        } else if self.moved_without_a_commit.contains(stuck) {
            format!(
                "{short} is still under it upstream, and its move out is not committed; \
                 run: dam add {short}, then dam commit and dam push"
            )
        } else {
            format!(
                "{short} is still under it upstream and this push does not move it; \
                 run: dam mv {short} {}, then dam push",
                self.hint.place_for(stuck)
            )
        }
    }
}

pub(super) fn named_by(stuck: &Oid, holding: &Holding) -> Vec<Oid> {
    let mut named = vec![stuck.clone()];
    if let Holding::MaybeHeld(could_hold) = holding {
        named.extend(could_hold.iter().cloned());
    }
    named
}
