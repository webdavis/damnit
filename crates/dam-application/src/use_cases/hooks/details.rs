use std::collections::BTreeMap;

use dam_domain::{Channel, Details, Oid};

use crate::config::HookEvent;
use crate::errors::UseCaseError;
use crate::service::Dam;

impl Dam<'_> {
    pub(crate) fn hook_details(&self, event: HookEvent) -> Details {
        let owners = self
            .access
            .details()
            .filter(|_| self.access.token().is_none());
        let mut details = owners.cloned().unwrap_or_else(|| Details {
            meta: BTreeMap::new(),
            host: None,
            dir: None,
            repo: None,
            branch: None,
            at: self.clock.now(),
            channel: Channel::Local,
        });
        details
            .meta
            .insert("hook".to_string(), event.as_str().to_string());
        details
    }

    pub(crate) fn staging_details(&self, oid: &Oid) -> Result<Option<Details>, UseCaseError> {
        Ok(match self.store.hooked(oid)? {
            Some(event) => Some(self.hook_details(event)),
            None => self.access.details().cloned(),
        })
    }
}
