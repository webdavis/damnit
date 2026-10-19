use std::collections::BTreeSet;

use dam_domain::{CommitId, CommitRecord, Object, Oid};

use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Settlement {
    pub(super) map: Vec<(Oid, String)>,
    pub(super) clear: Vec<Oid>,
    pub(super) snapshot: Vec<Object>,
    pub(super) failed: Vec<(Oid, String)>,
    pub(super) unanswered: BTreeSet<Oid>,
}

impl Settlement {
    pub(super) fn retries(&self) -> Vec<Oid> {
        self.failed.iter().map(|(oid, _)| oid.clone()).collect()
    }

    pub(super) fn pushed<'a>(
        &'a self,
        unpushed: &'a [CommitRecord],
    ) -> impl Iterator<Item = &'a CommitId> {
        unpushed
            .iter()
            .filter(|r| !r.changes.iter().any(|c| self.unanswered.contains(c.oid())))
            .map(|r| &r.id)
    }

    fn settle(&mut self, mutation: &RemoteMutation, verdict: MutationVerdict) {
        let oid = mutation.oid.clone();
        match verdict {
            MutationVerdict::Accepted { remote_id } => {
                if let Some(id) = remote_id {
                    self.map.push((oid.clone(), id));
                }
                match (&mutation.op, &mutation.object) {
                    (MutationOp::Delete, _) => self.clear.push(oid),
                    (_, Some(after)) => self.snapshot.push(after.clone()),
                    (_, None) => {}
                }
            }
            MutationVerdict::Refused { why } => self.failed.push((oid, why)),
        }
    }
}

pub(super) fn reconcile(sent: &[RemoteMutation], outcomes: Vec<MutationOutcome>) -> Settlement {
    let mut settlement = Settlement::default();
    let mut answered: BTreeSet<&Oid> = BTreeSet::new();
    for outcome in outcomes {
        let Some(mutation) = sent.iter().find(|m| m.oid == outcome.oid) else {
            continue;
        };
        if !answered.insert(&mutation.oid) {
            continue;
        }
        settlement.settle(mutation, outcome.verdict);
    }
    for mutation in sent.iter().filter(|m| !answered.contains(&m.oid)) {
        let oid = mutation.oid.clone();
        settlement
            .failed
            .push((oid.clone(), "no answer from helper".to_string()));
        settlement.unanswered.insert(oid);
    }
    settlement
}
