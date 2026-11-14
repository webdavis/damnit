use std::collections::BTreeSet;

use dam_domain::{CommitId, CommitRecord, Field, Object, Oid};

use crate::remote::{MutationOp, MutationOutcome, MutationVerdict, RemoteMutation};

const NO_ANSWER: &str = "no answer from helper";

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Settlement {
    pub(super) map: Vec<(Oid, String)>,
    pub(super) clear: Vec<Oid>,
    pub(super) snapshot: Vec<Object>,
    pub(super) failed: Vec<(Oid, String)>,
    pub(super) unanswered: BTreeSet<Oid>,
    pub(super) answered: Vec<Oid>,
    pub(super) doubted: Vec<(Oid, Vec<Field>)>,
    pub(super) confirmed: Vec<(Oid, Vec<Field>)>,
    pub(super) created: Vec<Oid>,
    pub(super) removed_in_doubt: Vec<Object>,
}

impl Settlement {
    pub(super) fn retries(&self) -> Vec<Oid> {
        self.failed.iter().map(|(oid, _)| oid.clone()).collect()
    }

    pub(super) fn absorb(&mut self, later: Settlement) {
        self.map.extend(later.map);
        self.clear.extend(later.clear);
        self.snapshot.extend(later.snapshot);
        self.failed.extend(later.failed);
        self.unanswered.extend(later.unanswered);
        self.answered.extend(later.answered);
        self.doubted.extend(later.doubted);
        self.confirmed.extend(later.confirmed);
        self.created.extend(later.created);
        self.removed_in_doubt.extend(later.removed_in_doubt);
    }

    pub(super) fn pushed<'a>(
        &'a self,
        unpushed: &'a [CommitRecord],
    ) -> impl Iterator<Item = &'a CommitId> {
        unpushed
            .iter()
            .filter(|r| {
                !r.changes
                    .iter()
                    .any(|c| self.unanswered.contains(c.change.oid()))
            })
            .map(|r| &r.id)
    }

    fn settle(&mut self, mutation: &RemoteMutation, verdict: MutationVerdict) {
        let oid = mutation.oid.clone();
        if verdict.is_an_answer_from_the_remote() {
            self.answered.push(oid.clone());
        }
        match verdict {
            MutationVerdict::Accepted { remote_id } => {
                match mutation.op {
                    MutationOp::Create => self.created.push(oid.clone()),
                    MutationOp::Update => {
                        self.confirmed.push((oid.clone(), mutation.fields.clone()))
                    }
                    MutationOp::Delete => {}
                }
                if let Some(id) = remote_id {
                    self.map.push((oid.clone(), id));
                }
                match (&mutation.op, &mutation.object) {
                    (MutationOp::Delete, _) => self.clear.push(oid),
                    (_, Some(after)) => self.snapshot.push(after.clone()),
                    (_, None) => {}
                }
            }
            MutationVerdict::Refused { why, .. } => self.failed.push((oid, why)),
            MutationVerdict::Unanswered { why } => self.unanswered(mutation, why),
        }
    }

    pub(super) fn never_sent(pending: &[RemoteMutation]) -> Settlement {
        let mut settlement = Settlement::default();
        for mutation in pending {
            settlement.owed_again(&mutation.oid, NO_ANSWER.to_string());
        }
        settlement
    }

    fn unanswered(&mut self, mutation: &RemoteMutation, why: String) {
        self.owed_again(&mutation.oid, why);
        self.doubted
            .push((mutation.oid.clone(), mutation.fields.clone()));
    }

    fn owed_again(&mut self, oid: &Oid, why: String) {
        self.failed.push((oid.clone(), why));
        self.unanswered.insert(oid.clone());
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
        settlement.unanswered(mutation, NO_ANSWER.to_string());
    }
    settlement
}
