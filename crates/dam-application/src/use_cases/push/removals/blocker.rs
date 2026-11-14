use std::collections::BTreeSet;

use dam_domain::{Oid, Path};

use super::Removals;
use super::release::Unreleased;
use crate::remote::{MutationOp, RemoteMutation};

pub(super) enum Blocker {
    StillToBeSent,
    Stuck(String),
}

pub(super) struct Unsent {
    oids: BTreeSet<Oid>,
    landings: Vec<(Oid, Path)>,
    owned_folders: Vec<Path>,
}

impl Unsent {
    pub(super) fn of(pending: &[RemoteMutation]) -> Unsent {
        Unsent {
            oids: pending.iter().map(|m| m.oid.clone()).collect(),
            landings: pending
                .iter()
                .filter_map(|m| Some((m.oid.clone(), m.object.as_ref()?.base().path.clone())))
                .collect(),
            owned_folders: pending
                .iter()
                .filter_map(|m| m.object.as_ref()?.base().owned_folder())
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
            .filter(|(_, held)| held.path.is_within(folder))
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
        let still_to_be_sent = under.iter().any(|oid| unsent.oids.contains(*oid));
        let Some(stuck) = under.into_iter().find(|oid| !unsent.oids.contains(*oid)) else {
            return Some(Blocker::StillToBeSent);
        };
        if unsent.owned_folders.contains(folder) {
            return Some(Blocker::StillToBeSent);
        }
        match self.namesake(&mutation.oid, folder) {
            Some(_) if self.remote_removes_only_empty_containers && still_to_be_sent => {
                Some(Blocker::StillToBeSent)
            }
            Some(namesake) if self.remote_removes_only_empty_containers => {
                let stuck: Vec<&Oid> = self.stuck_under(folder, unsent).collect();
                let unreleased = stuck.into_iter().find_map(|o| self.unreleased(o, folder))?;
                Some(Blocker::Stuck(match unreleased {
                    Unreleased::LeftUnplaced(o) => same_name(o, namesake),
                    Unreleased::MovedWithoutTellingUpstream(o) => self.why_held(o),
                }))
            }
            Some(namesake) => Some(Blocker::Stuck(same_name(stuck, namesake))),
            None => Some(Blocker::Stuck(self.why_held(stuck))),
        }
    }

    fn stuck_under<'a>(
        &'a self,
        folder: &'a Path,
        unsent: &'a Unsent,
    ) -> impl Iterator<Item = &'a Oid> {
        self.upstream
            .iter()
            .filter(move |(oid, held)| held.path.is_within(folder) && !unsent.oids.contains(*oid))
            .map(|(oid, _)| oid)
    }

    fn namesake(&self, removed: &Oid, folder: &Path) -> Option<&Oid> {
        self.upstream
            .iter()
            .filter(|(oid, _)| *oid != removed && !self.folders.contains_key(*oid))
            .find(|(_, held)| held.owned_folder().as_ref() == Some(folder))
            .map(|(oid, _)| oid)
    }

    pub(super) fn why_held(&self, stuck: &Oid) -> String {
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

fn same_name(stuck: &Oid, namesake: &Oid) -> String {
    let (stuck, namesake) = (stuck.short(), namesake.short());
    format!(
        "{namesake} has the same name in the same place, and dam cannot tell which of the two \
         holds {stuck} upstream; rename {namesake}, then push"
    )
}
