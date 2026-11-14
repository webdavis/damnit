use dam_domain::{ItemHash, Oid, Outcome};

use super::refusal::ItemRefusal;
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::use_cases::atomically::atomically;
use crate::use_cases::prefix::Named;

#[derive(Debug, PartialEq, Eq)]
pub enum Reopened {
    Unstaged(Oid),
    Waiting(ItemHash),
}

impl Dam<'_> {
    pub fn resolve_resettable(&self, text: &str) -> Result<Named, UseCaseError> {
        let named = self.resolve_named(text);
        match &named {
            Ok(Named::Item(_)) | Err(UseCaseError::NotAnOid(_)) => return named,
            Ok(Named::Object(oid)) if self.is_a_committed_new_item(oid)? => {
                return Ok(Named::Item(ItemHash::new(oid.clone())));
            }
            Ok(Named::Object(oid)) if self.store.committed(oid)?.is_some() => return named,
            Ok(Named::Object(oid)) if self.store.get(oid)?.is_some() => return named,
            _ => {}
        }
        let settled: Vec<ItemHash> = self
            .store
            .items()?
            .into_iter()
            .filter(|item| !item.is_waiting() && item.hash.oid().as_str().starts_with(text))
            .map(|item| item.hash)
            .collect();
        match settled.as_slice() {
            [] => named,
            [one] => Ok(Named::Item(one.clone())),
            _ => Err(Refusal::AmbiguousPrefix {
                text: text.to_string(),
                matches: settled.iter().map(ItemHash::to_string).collect(),
            }
            .into()),
        }
    }

    fn is_a_committed_new_item(&self, oid: &Oid) -> Result<bool, UseCaseError> {
        let added = self
            .store
            .item(&ItemHash::new(oid.clone()))?
            .is_some_and(|item| matches!(item.outcome, Outcome::Added(_)) && self.sees_item(&item));
        let staged = self.store.staged()?.iter().any(|s| s.change.oid() == oid);
        Ok(added && !staged)
    }

    pub fn reopen(&self, hash: &ItemHash) -> Result<Reopened, UseCaseError> {
        atomically(self.store, || {
            let item = self
                .store
                .item(hash)?
                .ok_or_else(|| Refusal::NoSuchObject(hash.short().to_string()))?;
            match item.outcome {
                Outcome::Declined(_) => {
                    self.store.settle(hash, Outcome::Waiting)?;
                    Ok(Reopened::Waiting(item.hash))
                }
                Outcome::Added(_) => {
                    let still_staged = self
                        .added_item_staged_at(item.target())?
                        .is_some_and(|staged| staged.hash == item.hash);
                    if !still_staged {
                        return Err(ItemRefusal::AlreadyCommitted(item.hash).into());
                    }
                    self.reset(std::slice::from_ref(item.target()))?;
                    Ok(Reopened::Unstaged(item.target().clone()))
                }
                Outcome::Waiting => Err(ItemRefusal::NotAnObject(item.hash).into()),
                Outcome::Withdrawn(_) => Err(ItemRefusal::NotWaiting {
                    hash: item.hash,
                    outcome: item.outcome.as_str(),
                }
                .into()),
            }
        })
    }
}

#[cfg(test)]
mod tests;
