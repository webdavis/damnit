use dam_domain::{CommitRecord, ItemHash, Oid};

use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::items::refusal::ItemRefusal;

pub const SHORTEST_OID_PREFIX: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Named {
    Object(Oid),
    Item(ItemHash),
}

impl Named {
    fn id(&self) -> &Oid {
        match self {
            Named::Object(oid) => oid,
            Named::Item(hash) => hash.oid(),
        }
    }
}

impl Dam<'_> {
    pub fn resolve_object(&self, text: &str) -> Result<Oid, UseCaseError> {
        match self.resolve_named(text)? {
            Named::Object(oid) => Ok(oid),
            Named::Item(hash) => Err(ItemRefusal::NotAnObject(hash).into()),
        }
    }

    pub fn resolve_named(&self, text: &str) -> Result<Named, UseCaseError> {
        let waiting = self
            .waiting_items()?
            .into_iter()
            .map(|i| Named::Item(i.hash));
        if let Ok(oid) = Oid::parse(text) {
            return self.named_in_full(oid);
        }
        require_a_hex_prefix(text)?;
        let objects = self
            .store
            .all()?
            .into_iter()
            .filter(|o| self.access.sees(o))
            .map(|o| Named::Object(o.oid().clone()));
        let matches: Vec<Named> = objects
            .chain(waiting)
            .filter(|named| named.id().as_str().starts_with(text))
            .collect();
        match matches.as_slice() {
            [] => Err(Refusal::NoWorkingObject(text.to_string()).into()),
            [one] => Ok(one.clone()),
            _ => Err(Refusal::AmbiguousPrefix {
                text: text.to_string(),
                matches: matches.iter().map(|named| named.id().to_string()).collect(),
            }
            .into()),
        }
    }

    fn named_in_full(&self, oid: Oid) -> Result<Named, UseCaseError> {
        let hash = ItemHash::new(oid.clone());
        let item = self.store.item(&hash)?;
        Ok(match item {
            Some(item) if item.is_waiting() && self.sees_item(&item) => Named::Item(hash),
            _ => Named::Object(oid),
        })
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
mod access_tests;
#[cfg(test)]
mod tests;
