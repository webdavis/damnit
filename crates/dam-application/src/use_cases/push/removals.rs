use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, Oid, Path};

use crate::errors::UseCaseError;
use crate::ports::{RemoteName, Store};
use crate::remote::RemoteMutation;
use crate::use_cases::push::mutations::OwedChanges;
use crate::use_cases::push::reconcile::Settlement;
use blocker::{Blocker, Unsent};

#[derive(Default)]
pub(super) struct Removals {
    folders: BTreeMap<Oid, Path>,
    upstream: BTreeMap<Oid, Path>,
    tried: BTreeSet<Oid>,
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
        changes: &OwedChanges,
    ) -> Result<Removals, UseCaseError> {
        let mut folders = BTreeMap::new();
        let removed = changes
            .iter()
            .filter(|(_, (change, _))| matches!(change, Change::Delete { .. }));
        for (oid, (change, _)) in removed {
            let held_at = store.remote_snapshot(remote, oid)?;
            let held_at = held_at.as_ref().or(change.before());
            if let Some(folder) = held_at.and_then(|o| o.base().owned_folder()) {
                folders.insert(oid.clone(), folder);
            }
        }
        let mut upstream = BTreeMap::new();
        if !folders.is_empty() {
            let local = store.all()?.into_iter().map(|o| o.oid().clone());
            let known: BTreeSet<Oid> = local.chain(changes.keys().cloned()).collect();
            for oid in known {
                if let Some(held) = store.remote_snapshot(remote, &oid)? {
                    upstream.insert(oid, held.base().path.clone());
                }
            }
        }
        Ok(Removals {
            folders,
            upstream,
            tried: BTreeSet::new(),
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
        for placed in &settled.snapshot {
            self.upstream
                .insert(placed.oid().clone(), placed.base().path.clone());
        }
        for removed in &settled.clear {
            self.upstream.remove(removed);
        }
    }
}

mod blocker;

#[cfg(test)]
mod tests;
