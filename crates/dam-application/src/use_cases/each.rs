use std::collections::BTreeMap;

use dam_domain::{Change, Oid, Path};

use crate::config::FilterConfig;
use crate::errors::{Refusal, UseCaseError};
use crate::ports::Step;
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::items::land::{Landed, Sent};
use rehearsal::rehearsed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    Oids(Vec<Oid>),
    Filter(String, Path, Vec<FilterConfig>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SetRun {
    pub preview: bool,
    pub expect: Option<usize>,
    pub as_one: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetChanges {
    pub preview: bool,
    pub oids: Vec<Oid>,
    pub changes: Vec<SetChange>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetChange {
    pub change: Change,
    pub sent: Option<Sent>,
}

impl Dam<'_> {
    pub(crate) fn each(
        &self,
        selector: &Selector,
        run: SetRun,
        completing: bool,
        mut verb: impl FnMut(&Oid) -> Result<Option<Sent>, UseCaseError>,
    ) -> Result<SetChanges, UseCaseError> {
        let oids = self.select(selector)?;
        if let Some(expected) = run.expect
            && expected != oids.len()
        {
            return Err(Refusal::CountMismatch {
                expected,
                count: oids.len(),
            }
            .into());
        }
        let mut work = || self.changing(&oids, &mut verb);
        let changes = if run.preview {
            rehearsed(self.store, work)?
        } else {
            let completing = if completing { &oids[..] } else { &[] };
            atomically(self.store, || {
                let changes = self.entering(completing, &mut work)?;
                match run.as_one {
                    true => self.gathered(changes),
                    false => Ok(changes),
                }
            })?
        };
        Ok(SetChanges {
            preview: run.preview,
            oids,
            changes,
        })
    }

    fn changing(
        &self,
        oids: &[Oid],
        verb: &mut dyn FnMut(&Oid) -> Result<Option<Sent>, UseCaseError>,
    ) -> Result<Vec<SetChange>, UseCaseError> {
        let mut before = BTreeMap::new();
        for oid in oids {
            before.insert(oid.clone(), self.store.get(oid)?);
        }
        let mut sent = BTreeMap::new();
        let progress = self.shown_progress();
        progress.start(Step {
            name: "change",
            remote: None,
            doing: "changing",
            did: "changed",
            unit: "objects",
            total: Some(oids.len()),
        });
        for (done, oid) in oids.iter().enumerate() {
            match verb(oid) {
                Ok(Some(item)) => {
                    sent.insert(oid.clone(), item);
                }
                Ok(None) => {}
                Err(e) => {
                    progress.fail();
                    return Err(e);
                }
            }
            progress.advance(done + 1);
        }
        progress.finish();
        let mut changes = Vec::new();
        for oid in oids {
            let found = match sent.remove(oid) {
                Some(sent) => Some(SetChange {
                    change: sent.item.change.clone(),
                    sent: Some(sent),
                }),
                None => Change::between(before.remove(oid).flatten(), self.store.get(oid)?)
                    .map(|change| SetChange { change, sent: None }),
            };
            if let Some(mut found) = found {
                found.change = self.as_seen_change(found.change)?;
                changes.push(found);
            }
        }
        Ok(changes)
    }
}

pub(crate) fn sent(landed: Landed) -> Option<Sent> {
    match landed {
        Landed::Sent(sent) => Some(sent),
        Landed::Written(_) => None,
    }
}

mod as_one;
pub(crate) mod rehearsal;
mod select;
mod verbs;

#[cfg(test)]
mod tests;
