use dam_domain::{Change, Object, Oid};

use super::PullReport;
use super::classify::Incoming;
use super::notices::note_cancelled;
use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Store;
use crate::service::Dam;
use crate::use_cases::commit::record;

impl Dam<'_> {
    pub(super) fn land(
        &self,
        remote: &RemoteConfig,
        planned: Vec<(Oid, Incoming)>,
        report: &mut PullReport,
    ) -> Result<(), UseCaseError> {
        let mut landed = Vec::new();
        for (oid, incoming) in planned {
            match incoming {
                Incoming::Unchanged => report.unchanged += 1,
                Incoming::Create(o) => {
                    landed.extend(take_theirs(self.store, remote, &oid, o)?);
                    report.created += 1;
                }
                Incoming::FastForward(o) => {
                    landed.extend(take_theirs(self.store, remote, &oid, o)?);
                    report.updated += 1;
                }
                Incoming::Conflict(conflict) => {
                    self.store.mark_conflict(&conflict)?;
                    self.store
                        .set_remote_snapshot(&remote.name, &conflict.theirs)?;
                    report.conflicts += 1;
                    note_cancelled(self.store, &conflict.theirs)?;
                }
            }
        }
        if landed.is_empty() {
            return Ok(());
        }
        self.commit_as_already_pushed(remote, landed)
    }

    fn commit_as_already_pushed(
        &self,
        remote: &RemoteConfig,
        landed: Vec<Change>,
    ) -> Result<(), UseCaseError> {
        let pulled = record(
            self.clock,
            self.random,
            format!("pull from {}", remote.name.0),
            landed,
        );
        self.store.commit(&pulled)?;
        self.store.mark_pushed(&remote.name, &pulled.id)?;
        Ok(())
    }
}

fn take_theirs(
    store: &dyn Store,
    remote: &RemoteConfig,
    oid: &Oid,
    theirs: Object,
) -> Result<Option<Change>, UseCaseError> {
    store.put(&theirs)?;
    store.set_remote_snapshot(&remote.name, &theirs)?;
    let committed = store.committed(oid)?;
    note_cancelled(store, &theirs)?;
    Ok(Change::between(committed, Some(theirs)))
}
