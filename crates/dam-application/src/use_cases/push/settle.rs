use dam_domain::{CommitRecord, Oid};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Notice;
use crate::service::Dam;
use crate::use_cases::push::doubt::settle_doubt;
use crate::use_cases::push::reconcile::Settlement;

impl Dam<'_> {
    pub(super) fn settle_push(
        &self,
        remote: &RemoteConfig,
        unpushed: &[CommitRecord],
        settlement: &Settlement,
        earlier_retries: &[Oid],
    ) -> Result<(), UseCaseError> {
        let store = self.store;
        for (oid, id) in &settlement.map {
            store.map_remote_id(&remote.name, oid, id)?;
        }
        for oid in &settlement.clear {
            store.clear_remote_mapping(&remote.name, oid)?;
        }
        for after in &settlement.snapshot {
            store.set_remote_snapshot(&remote.name, after)?;
        }
        for (oid, why) in &settlement.failed {
            store.add_notice(&Notice::PushFailed {
                remote: remote.name.clone(),
                oid: oid.clone(),
                why: why.clone(),
            })?;
        }
        for removed in &settlement.removed_in_doubt {
            let base = removed.base();
            store.add_notice(&Notice::CreateInDoubtRemoved {
                remote: remote.name.clone(),
                oid: base.oid.clone(),
                subject: base.subject.clone(),
                path: base.path.clone(),
            })?;
        }
        for oid in &settlement.answered {
            store.count_answer(&remote.name, oid)?;
        }
        settle_doubt(store, &remote.name, settlement)?;
        let mut retries = earlier_retries.to_vec();
        retries.extend(settlement.retries());
        store.set_push_retries(&remote.name, &retries)?;
        for id in settlement.pushed(unpushed) {
            store.mark_pushed(&remote.name, id)?;
        }
        store.set_last_push(&remote.name, self.clock.now())?;
        Ok(())
    }
}
