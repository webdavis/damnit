use std::collections::HashSet;

use dam_domain::Permission;

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::RemoteName;
use crate::remote::{PullOutcome, RemoteCapabilities};
use crate::service::{Dam, Remotes};
use crate::use_cases::connect::connect;

mod cancelled;
mod classify;
mod land;
mod missing;
mod notices;
mod skipped;

use classify::classify;
use missing::record_missing;
use notices::record_removals;
use skipped::record_skipped;

#[derive(Debug)]
pub struct PullReport {
    pub remote: RemoteName,
    pub created: usize,
    pub updated: usize,
    pub conflicts: usize,
    pub removed_upstream: usize,
    pub unchanged: usize,
}

impl Dam<'_> {
    pub fn pull(
        &self,
        remotes: Remotes<'_>,
        remote: Option<&str>,
    ) -> Result<Vec<PullReport>, UseCaseError> {
        let mut reports = Vec::new();
        for target in self.remotes_permitting(Permission::RemotePull, remote)? {
            let (mut helper, caps) = connect(remotes, self.store, self.access, target)?;
            let since = self.store.sync_token(&target.name)?;
            let response = helper.pull(since.as_deref())?;
            reports.push(self.apply_pulled(target, &caps, response)?);
        }
        Ok(reports)
    }

    fn apply_pulled(
        &self,
        remote: &RemoteConfig,
        caps: &RemoteCapabilities,
        mut response: PullOutcome,
    ) -> Result<PullReport, UseCaseError> {
        let store = self.store;
        let mut report = PullReport {
            remote: remote.name.clone(),
            created: 0,
            updated: 0,
            conflicts: 0,
            removed_upstream: 0,
            unchanged: 0,
        };
        let listed: HashSet<String> = response
            .objects
            .iter()
            .map(|o| o.remote_id.clone())
            .chain(response.rejected.iter().map(|r| r.remote_id.clone()))
            .collect();
        let mut land_everything_as_one_unit = || {
            record_skipped(store, &remote.name, &response.objects, &response.rejected)?;
            let mut objects = std::mem::take(&mut response.objects);
            objects.extend(cancelled::cancelled_events(
                store,
                remote,
                &response.cancelled,
            )?);
            let (planned, new_remote_id_mappings) =
                classify(store, remote, caps, objects, self.random)?;
            for (oid, remote_id) in new_remote_id_mappings {
                store.map_remote_id(&remote.name, &oid, &remote_id)?;
            }
            self.land(remote, planned, &mut report)?;
            let removed = std::mem::take(&mut response.removed);
            report.removed_upstream = record_removals(store, remote, removed)?;
            if response.complete {
                record_missing(store, remote, &listed)?;
            }
            store.set_sync_token(&remote.name, response.sync.as_deref())?;
            store.set_last_pull(&remote.name, self.clock.now())?;
            Ok(())
        };
        store.in_transaction(&mut land_everything_as_one_unit)?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests;
