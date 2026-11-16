use std::collections::BTreeSet;

use dam_domain::{Object, Oid, Path, nearest_parents};

use super::Removals;
use crate::remote::{MutationOp, RemoteMutation};

pub(super) enum Blocker {
    StillToBeSent,
    Stuck(String),
}

pub(super) struct Unsent {
    oids: BTreeSet<Oid>,
    landings: Vec<(Oid, Option<Oid>, Path)>,
}

enum Holding {
    Held,
    MaybeHeld(Vec<Oid>),
    PlaceNotKnown,
}

impl Unsent {
    pub(super) fn of(pending: &[RemoteMutation]) -> Unsent {
        Unsent {
            oids: pending.iter().map(|m| m.oid.clone()).collect(),
            landings: pending
                .iter()
                .filter_map(|m| {
                    let base = m.object.as_ref()?.base();
                    Some((m.oid.clone(), base.parent.clone(), base.path.clone()))
                })
                .collect(),
        }
    }
}

impl Removals {
    pub(super) fn blocker(&self, mutation: &RemoteMutation, unsent: &Unsent) -> Option<Blocker> {
        if mutation.op != MutationOp::Delete || !self.removing.contains(&mutation.oid) {
            return None;
        }
        let removed = &mutation.oid;
        if !self.upstream.contains_key(removed) {
            return Some(Blocker::Stuck(format!(
                "what {} holds upstream is not known; run: dam pull, then dam push",
                removed.short()
            )));
        }
        let held_under = self
            .upstream
            .values()
            .filter(|held| held.oid() != removed)
            .filter_map(|held| Some((held.oid(), self.holding(held, removed)?)));
        let folder = self
            .upstream
            .get(removed)
            .and_then(|o| o.base().owned_folder());
        let landing_under = unsent
            .landings
            .iter()
            .filter(|(_, parent, path)| match parent {
                Some(parent) => parent == removed,
                None => folder.as_ref().is_none_or(|folder| path.is_within(folder)),
            })
            .map(|(oid, ..)| (oid, Holding::Held));
        let under: Vec<(&Oid, Holding)> = held_under.chain(landing_under).collect();
        if under.is_empty() {
            return None;
        }
        match under.iter().find(|(oid, _)| !unsent.oids.contains(*oid)) {
            None => Some(Blocker::StillToBeSent),
            Some((stuck, holding)) => Some(Blocker::Stuck(self.why_held(stuck, holding))),
        }
    }

    fn holding(&self, held: &Object, removed: &Oid) -> Option<Holding> {
        let mut at = held;
        for _ in 0..=self.upstream.len() {
            match &at.base().parent {
                Some(parent) if parent == removed => return Some(Holding::Held),
                Some(parent) => match self.upstream.get(parent) {
                    Some(next) => at = next,
                    None => return self.inside_the_folder_of(removed, &held.base().path),
                },
                None => {
                    let could_hold = nearest_parents(&at.base().path, self.upstream.values());
                    return match could_hold.as_slice() {
                        [only] if only == removed => Some(Holding::Held),
                        several if several.contains(removed) => {
                            Some(Holding::MaybeHeld(could_hold))
                        }
                        _ => None,
                    };
                }
            }
        }
        Some(Holding::PlaceNotKnown)
    }

    fn inside_the_folder_of(&self, removed: &Oid, path: &Path) -> Option<Holding> {
        let folder = self
            .upstream
            .get(removed)
            .and_then(|o| o.base().owned_folder());
        match folder {
            Some(folder) if !path.is_within(&folder) => None,
            _ => Some(Holding::PlaceNotKnown),
        }
    }

    fn why_held(&self, stuck: &Oid, holding: &Holding) -> String {
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
