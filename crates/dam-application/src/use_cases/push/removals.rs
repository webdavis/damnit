use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, Object, Oid};

use crate::errors::UseCaseError;
use crate::ports::{RemoteName, Store};
use crate::remote::RemoteMutation;
use crate::use_cases::last_seen::LastSeen;
use crate::use_cases::push::owed::OwedChanges;
use crate::use_cases::push::reconcile::Settlement;
use blocker::{Blocker, Unsent};

#[derive(Default)]
pub(super) struct Removals {
    removing: BTreeSet<Oid>,
    upstream: BTreeMap<Oid, Object>,
    moved_without_a_commit: BTreeSet<Oid>,
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
        let mut removals = Removals {
            removing: changes
                .iter()
                .filter(|(_, change)| matches!(change, Change::Delete { .. }))
                .map(|(oid, _)| oid.clone())
                .collect(),
            ..Removals::default()
        };
        if removals.removing.is_empty() {
            return Ok(removals);
        }
        let kept = store.all()?.into_iter().map(|o| o.oid().clone());
        let known: BTreeSet<Oid> = kept.chain(changes.keys().cloned()).collect();
        for oid in known {
            if moved_since_its_commit(store.get(&oid)?, store.committed(&oid)?) {
                removals.moved_without_a_commit.insert(oid.clone());
            }
            if let Some(held) = LastSeen::held(store, remote, &oid)? {
                removals.upstream.insert(oid, held);
            }
        }
        Ok(removals)
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
            self.upstream.insert(placed.oid().clone(), placed.clone());
        }
        for removed in &settled.clear {
            self.upstream.remove(removed);
        }
    }
}

fn moved_since_its_commit(working: Option<Object>, committed: Option<Object>) -> bool {
    let place = |o: &Object| (o.base().path.clone(), o.base().parent.clone());
    matches!((working, committed), (Some(w), Some(c)) if place(&w) != place(&c))
}

mod blocker;

#[cfg(test)]
mod tests;
