use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Base, Change, Object, Oid, Path};

use crate::errors::UseCaseError;
use crate::ports::{RemoteName, Store};
use crate::remote::{RemoteCapabilities, RemoteMutation};
use crate::use_cases::push::owed::OwedChanges;
use crate::use_cases::push::reconcile::Settlement;
use blocker::{Blocker, Unsent};

#[derive(Default)]
pub(super) struct Removals {
    folders: BTreeMap<Oid, Path>,
    upstream: BTreeMap<Oid, Base>,
    local: BTreeMap<Oid, Path>,
    placed: BTreeSet<Oid>,
    tried: BTreeSet<Oid>,
    remote_removes_only_empty_containers: bool,
}

#[derive(Default)]
pub(super) struct Wave {
    pub(super) ready: Vec<RemoteMutation>,
    pub(super) waiting: Vec<RemoteMutation>,
    pub(super) held: Vec<(Oid, String)>,
}

const NOTHING_IT_WAITS_ON_WAS_SENT: &str =
    "waits on other removals in this push, and none of them could be sent";

impl Removals {
    pub(super) fn new(
        store: &dyn Store,
        remote: &RemoteName,
        caps: &RemoteCapabilities,
        changes: &OwedChanges,
    ) -> Result<Removals, UseCaseError> {
        let mut folders = BTreeMap::new();
        let removed = changes
            .iter()
            .filter(|(_, change)| matches!(change, Change::Delete { .. }));
        for (oid, change) in removed {
            let held_at = store.remote_snapshot(remote, oid)?;
            let held_at = held_at.as_ref().or(change.before());
            if let Some(folder) = held_at.and_then(|o| o.base().owned_folder()) {
                folders.insert(oid.clone(), folder);
            }
        }
        let mut upstream = BTreeMap::new();
        let mut local = BTreeMap::new();
        if !folders.is_empty() {
            let kept = store.all()?.into_iter().map(|o| o.oid().clone());
            let known: BTreeSet<Oid> = kept.chain(changes.keys().cloned()).collect();
            for oid in known {
                if let Some(committed) = store.committed(&oid)? {
                    local.insert(oid.clone(), committed.base().path.clone());
                }
                if let Some(held) = store.remote_snapshot(remote, &oid)? {
                    upstream.insert(oid, held.base().clone());
                }
            }
        }
        Ok(Removals {
            folders,
            upstream,
            local,
            placed: BTreeSet::new(),
            tried: BTreeSet::new(),
            remote_removes_only_empty_containers: caps.removes_only_empty_containers,
        })
    }

    pub(super) fn wave(&mut self, pending: Vec<RemoteMutation>) -> Wave {
        let unsent = Unsent::of(&pending);
        let mut wave = Wave::default();
        for mutation in pending {
            match self.blocker(&mutation, &unsent) {
                None => wave.ready.push(mutation),
                Some(Blocker::StillToBeSent) => wave.waiting.push(mutation),
                Some(Blocker::Stuck(why)) => wave.held.push((mutation.oid, why)),
            }
        }
        if wave.ready.is_empty() && wave.held.is_empty() {
            let stalled = std::mem::take(&mut wave.waiting);
            wave.held = stalled
                .into_iter()
                .map(|m| (m.oid, NOTHING_IT_WAITS_ON_WAS_SENT.to_string()))
                .collect();
        }
        self.tried
            .extend(wave.held.iter().map(|(oid, _)| oid.clone()));
        wave
    }

    pub(super) fn record(&mut self, sent: &[RemoteMutation], settled: &Settlement) {
        self.tried.extend(sent.iter().map(|m| m.oid.clone()));
        let accepted: BTreeSet<&Oid> = settled.snapshot.iter().map(Object::oid).collect();
        let placed_by_this_push = sent
            .iter()
            .filter(|m| accepted.contains(&m.oid) && release::places(m))
            .map(|m| m.oid.clone());
        self.placed.extend(placed_by_this_push);
        for placed in &settled.snapshot {
            self.upstream
                .insert(placed.oid().clone(), placed.base().clone());
        }
        for removed in &settled.clear {
            self.upstream.remove(removed);
        }
    }
}

mod blocker;
mod release;

#[cfg(test)]
mod tests;
