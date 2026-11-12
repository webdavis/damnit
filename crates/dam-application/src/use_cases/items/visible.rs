use dam_domain::{Item, Permission};

use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub fn waiting_items(&self) -> Result<Vec<Item>, UseCaseError> {
        let mut seen = Vec::new();
        for mut item in self.store.items()? {
            if item.is_waiting() && self.sees_item(&item) {
                item.change = self.as_seen_change(item.change)?;
                seen.push(item);
            }
        }
        Ok(seen)
    }

    pub(crate) fn sees_item(&self, item: &Item) -> bool {
        if self.access.token() == Some(item.author.name.as_str()) {
            return true;
        }
        self.permits_change(Permission::PendingRead, &item.change)
    }
}

#[cfg(test)]
mod tests;
