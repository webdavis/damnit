use std::collections::BTreeSet;

use dam_domain::{Oid, Path};

use super::Removals;
use crate::remote::{MutationOp, RemoteMutation};

pub(super) enum Blocker {
    StillToBeSent,
    Stuck(String),
}

pub(super) struct Unsent {
    oids: BTreeSet<Oid>,
    landings: Vec<(Oid, Path)>,
}

impl Unsent {
    pub(super) fn of(pending: &[RemoteMutation]) -> Unsent {
        Unsent {
            oids: pending.iter().map(|m| m.oid.clone()).collect(),
            landings: pending
                .iter()
                .filter_map(|m| Some((m.oid.clone(), m.object.as_ref()?.base().path.clone())))
                .collect(),
        }
    }
}

impl Removals {
    pub(super) fn blocker(&self, mutation: &RemoteMutation, unsent: &Unsent) -> Option<Blocker> {
        if mutation.op != MutationOp::Delete {
            return None;
        }
        let folder = self.folders.get(&mutation.oid)?;
        let held_under = self
            .upstream
            .iter()
            .filter(|(_, path)| path.is_within(folder))
            .map(|(oid, _)| oid);
        let landing_under = unsent
            .landings
            .iter()
            .filter(|(_, path)| path.is_within(folder))
            .map(|(oid, _)| oid);
        let under: Vec<&Oid> = held_under.chain(landing_under).collect();
        if under.is_empty() {
            return None;
        }
        Some(
            match under.into_iter().find(|oid| !unsent.oids.contains(*oid)) {
                Some(stuck) => Blocker::Stuck(self.why_held(stuck)),
                None => Blocker::StillToBeSent,
            },
        )
    }

    fn why_held(&self, stuck: &Oid) -> String {
        let short = stuck.short();
        if self.tried.contains(stuck) {
            format!("waits on {short} leaving it upstream, which this push did not do")
        } else {
            format!(
                "{short} is still under it upstream and this push does not move it; \
                 move {short} back inside this remote's path, then push"
            )
        }
    }
}
