use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Authored, Change, Channel, CommitId, Details, Object, Oid};

use crate::config::HookEvent;
use crate::errors::UseCaseError;
use crate::ports::{RemoteName, Store};
use crate::service::Dam;
use crate::use_cases::commit::record;

pub(super) struct BeforeTheHook {
    working: BTreeMap<Oid, Object>,
    unfinished: BTreeSet<Oid>,
}

impl BeforeTheHook {
    pub(super) fn of(store: &dyn Store) -> Result<BeforeTheHook, UseCaseError> {
        let mut unfinished: BTreeSet<Oid> = store
            .staged()?
            .into_iter()
            .map(|staged| staged.change.oid().clone())
            .collect();
        let mut working = BTreeMap::new();
        for object in store.all()? {
            let oid = object.oid().clone();
            if store.committed(&oid)?.as_ref() != Some(&object) {
                unfinished.insert(oid.clone());
            }
            working.insert(oid, object);
        }
        Ok(BeforeTheHook {
            working,
            unfinished,
        })
    }
}

impl Dam<'_> {
    pub(super) fn commit_hook_edits(
        &self,
        remote: &RemoteName,
        before: &BeforeTheHook,
    ) -> Result<Option<CommitId>, UseCaseError> {
        let mut changes = Vec::new();
        for object in self.store.all()? {
            let oid = object.oid();
            if before.working.get(oid) == Some(&object) || before.unfinished.contains(oid) {
                continue;
            }
            changes.extend(Change::between(self.store.committed(oid)?, Some(object)));
        }
        if changes.is_empty() {
            return Ok(None);
        }
        let details = self.hook_details();
        let entries = changes
            .into_iter()
            .map(|change| Authored::by_owner(change).with_details(Some(&details)));
        let message = format!("task_pulled after pull from {}", remote.0);
        let commit = record(self.clock, self.random, message, entries.collect());
        self.store.commit(&commit)?;
        Ok(Some(commit.id))
    }

    fn hook_details(&self) -> Details {
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
        let hook = HookEvent::TaskPulled.as_str().to_string();
        details.meta.insert("hook".to_string(), hook);
        details
    }
}
