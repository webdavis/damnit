use dam_domain::{CommitRecord, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;

pub(crate) const SHORTEST_OID_PREFIX: usize = 4;

impl Dam<'_> {
    pub fn resolve_object(&self, text: &str) -> Result<Oid, UseCaseError> {
        if let Ok(oid) = Oid::parse(text) {
            return Ok(oid);
        }
        require_a_hex_prefix(text)?;
        let matches: Vec<Oid> = self
            .store
            .all()?
            .into_iter()
            .map(|o| o.oid().clone())
            .filter(|o| o.as_str().starts_with(text))
            .collect();
        match matches.as_slice() {
            [] => Err(Refusal::NoWorkingObject(text.to_string()).into()),
            [one] => Ok(one.clone()),
            _ => Err(Refusal::AmbiguousPrefix {
                text: text.to_string(),
                matches: matches.iter().map(Oid::to_string).collect(),
            }
            .into()),
        }
    }

    pub fn resolve_commit(&self, text: &str) -> Result<CommitRecord, UseCaseError> {
        require_a_hex_prefix(text)?;
        let mut matches: Vec<CommitRecord> = self
            .store
            .log()?
            .into_iter()
            .filter(|c| c.id.as_str().starts_with(text))
            .collect();
        match matches.len() {
            0 => Err(Refusal::NoSuchObject(text.to_string()).into()),
            1 => Ok(matches.remove(0)),
            _ => Err(Refusal::AmbiguousPrefix {
                text: text.to_string(),
                matches: matches.iter().map(|c| c.id.to_string()).collect(),
            }
            .into()),
        }
    }
}

fn require_a_hex_prefix(text: &str) -> Result<(), UseCaseError> {
    if text.len() < SHORTEST_OID_PREFIX || !text.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(UseCaseError::NotAnOid(text.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
