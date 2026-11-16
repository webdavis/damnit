use dam_domain::CommitRecord;

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
}

#[cfg(test)]
mod tests;
