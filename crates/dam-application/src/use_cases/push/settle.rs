use dam_domain::{CommitRecord, Oid};

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::Notice;
use crate::service::Dam;
use crate::use_cases::last_seen::LastSeen;
use crate::use_cases::push::attempting::Attempting;
use crate::use_cases::push::attempts::settle_attempts;
use crate::use_cases::push::failure_notices::renew_failure_notices;
use crate::use_cases::push::reconcile::{Heard, Settlement};

impl Dam<'_> {
    pub(super) fn settle_push(
        &self,
        remote: &RemoteConfig,
        unpushed: &[CommitRecord],
        settlement: &Settlement,
        attempting: &Attempting,
        earlier_retries: &[Oid],
    ) -> Result<(), UseCaseError> {
        let store = self.store;
        for (oid, id) in &settlement.map {
            store.map_remote_id(&remote.name, oid, id)?;
        }
        for oid in &settlement.clear {
            LastSeen::gone(store, &remote.name, oid)?;
        }
        for after in &settlement.snapshot {
            LastSeen::now_holds(store, &remote.name, after)?;
        }
        let answers = settlement
            .heard
            .iter()
            .filter(|(_, h)| *h != Heard::Unasked);
        for (oid, _) in answers {
            store.count_answer(&remote.name, oid)?;
        }
        settle_attempts(store, &remote.name, &settlement.heard, attempting)?;
        for (object, because) in &settlement.dropped.creates {
            let base = object.base();
            store.add_notice(&Notice::UnansweredCreateDropped {
                remote: remote.name.clone(),
                oid: base.oid.clone(),
                subject: base.subject.clone(),
                path: base.path.clone(),
                because: *because,
            })?;
            store.count_answer(&remote.name, &base.oid)?;
            store.set_attempts(&remote.name, &base.oid, &[])?;
        }
        for oid in &settlement.dropped.unused {
            store.count_answer(&remote.name, oid)?;
            store.set_attempts(&remote.name, oid, &[])?;
        }
        let mut retries = earlier_retries.to_vec();
        retries.extend(settlement.retries());
        store.set_push_retries(&remote.name, &retries)?;
        renew_failure_notices(store, &remote.name, &retries, &settlement.failed)?;
        for id in settlement.pushed(unpushed) {
            store.mark_pushed(&remote.name, id)?;
        }
        store.set_last_push(&remote.name, self.clock.now())?;
        Ok(())
    }
}
