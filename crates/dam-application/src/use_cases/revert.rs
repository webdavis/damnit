use dam_domain::{Field, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;

mod entries;
mod set_back;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Reverted {
    pub reverted: Vec<Fields>,
    pub kept: Vec<Fields>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Fields {
    pub oid: Oid,
    pub fields: Vec<Field>,
}

impl Dam<'_> {
    pub fn revert(&self, commit: &str, objects: &[String]) -> Result<Reverted, UseCaseError> {
        if self.access.token().is_some() {
            return Err(Refusal::NotOpenToTokens("revert").into());
        }
        let record = self.resolve_commit(commit)?;
        if record.pulled {
            return Err(Refusal::PulledCommit(record.id).into());
        }
        let changes = entries::named(&record, objects)?;
        atomically(self.store, || {
            let mut out = Reverted::default();
            for change in &changes {
                let split = set_back::set_back(self.store, change)?;
                out.add(change.oid(), split);
            }
            Ok(out)
        })
    }
}

impl Reverted {
    fn add(&mut self, oid: &Oid, (reverted, kept): (Vec<Field>, Vec<Field>)) {
        for (list, fields) in [(&mut self.reverted, reverted), (&mut self.kept, kept)] {
            if !fields.is_empty() {
                list.push(Fields {
                    oid: oid.clone(),
                    fields,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests;
