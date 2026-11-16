use dam_domain::{Change, CommitRecord, Object, Oid};

use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn seen_log(&self) -> Result<Vec<CommitRecord>, UseCaseError> {
        let log = self.store.log()?;
        if self.access.token().is_none() {
            return Ok(log);
        }
        let mut seen = Vec::new();
        for mut record in log {
            let all = record.changes.len();
            let mut shown = Vec::new();
            for mut authored in record.changes {
                if self.reads_change(&authored.change)? {
                    authored.change = self.as_seen_change(authored.change)?;
                    shown.push(authored);
                }
            }
            if shown.is_empty() {
                continue;
            }
            if shown.len() < all {
                record.message = String::new();
            }
            record.changes = shown;
            seen.push(record);
        }
        Ok(seen)
    }

    fn reads_change(&self, change: &Change) -> Result<bool, UseCaseError> {
        let every_state_seen = change
            .before()
            .into_iter()
            .chain(change.after())
            .all(|object| self.access.sees(object));
        Ok(every_state_seen && !self.hidden_now(change.oid())?)
    }

    fn hidden_now(&self, oid: &Oid) -> Result<bool, UseCaseError> {
        let unseen = |object: Option<Object>| object.is_some_and(|o| !self.access.sees(&o));
        Ok(unseen(self.store.get(oid)?) || unseen(self.store.committed(oid)?))
    }
}

#[cfg(test)]
mod tests;
