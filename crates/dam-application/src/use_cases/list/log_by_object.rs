use std::cell::OnceCell;
use std::collections::{BTreeMap, HashMap};

use dam_domain::{Authored, CommitRecord, Oid, Permission, Subject};

use crate::ports::{Histories, History, StoreError};
use crate::service::Dam;

pub(super) struct LogByObject<'d, 'a> {
    dam: &'d Dam<'a>,
    grouped: OnceCell<Result<HashMap<Oid, Vec<CommitRecord>>, StoreError>>,
}

impl<'d, 'a> LogByObject<'d, 'a> {
    pub(super) fn new(dam: &'d Dam<'a>) -> Self {
        LogByObject {
            dam,
            grouped: OnceCell::new(),
        }
    }

    pub(super) fn unread(self) -> Result<(), StoreError> {
        match self.grouped.into_inner() {
            Some(Err(error)) => Err(error),
            _ => Ok(()),
        }
    }

    fn grouped(&self) -> Result<HashMap<Oid, Vec<CommitRecord>>, StoreError> {
        let mut by_object: HashMap<Oid, Vec<CommitRecord>> = HashMap::new();
        for commit in self.dam.store.log()? {
            let mut changes_of: BTreeMap<Oid, Vec<Authored>> = BTreeMap::new();
            for change in &commit.changes {
                changes_of
                    .entry(change.change.oid().clone())
                    .or_default()
                    .push(self.as_the_log_shows_it(change));
            }
            for (oid, changes) in changes_of {
                by_object.entry(oid).or_default().push(CommitRecord {
                    id: commit.id.clone(),
                    message: commit.message.clone(),
                    at: commit.at,
                    changes,
                });
            }
        }
        Ok(by_object)
    }

    fn as_the_log_shows_it(&self, change: &Authored) -> Authored {
        let shown = match self.dam.access.token() {
            None => true,
            Some(reader) => change.author.token().is_some_and(|t| t.name == reader),
        };
        Authored {
            details: change.details.clone().filter(|_| shown),
            ..change.clone()
        }
    }
}

impl Histories for LogByObject<'_, '_> {
    fn of(&self, oid: &Oid) -> History {
        let open = self
            .dam
            .access
            .permits(Permission::HistoryRead, &[Subject::Whole]);
        if !open {
            return History::Withheld;
        }
        match self.grouped.get_or_init(|| self.grouped()) {
            Ok(by_object) => History::Recorded(by_object.get(oid).cloned().unwrap_or_default()),
            Err(_) => History::Unread,
        }
    }
}
