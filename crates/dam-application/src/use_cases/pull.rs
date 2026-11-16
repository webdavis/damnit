use std::collections::HashSet;

use dam_domain::Permission;

use crate::config::RemoteConfig;
use crate::errors::UseCaseError;
use crate::ports::{RemoteHelper, RemoteName, Step};
use crate::remote::{PullOutcome, RemoteCapabilities};
use crate::service::{Dam, Remotes};
use crate::use_cases::hooks::pulled::{PulledHookRun, PulledTask};

mod adopt;
mod cancelled;
mod classify;
mod compare;
mod land;
mod missing;
mod notices;
mod ours;
mod removal;
mod removed;
mod skipped;
mod untouched;

use classify::classify;
use missing::record_missing;
use removed::record_removals;
use skipped::record_skipped;

#[derive(Debug)]
pub struct PullReport {
    pub remote: RemoteName,
    pub created: usize,
    pub updated: usize,
    pub conflicts: usize,
    pub removed_upstream: usize,
    pub unchanged: usize,
    pub creates_matched: usize,
    pub values_kept: usize,
    pub deletes_confirmed: usize,
    pub task_pulled: Option<PulledHookRun>,
}

impl Dam<'_> {
    pub fn pull(
        &self,
        remotes: Remotes<'_>,
        remote: Option<&str>,
    ) -> Result<Vec<PullReport>, UseCaseError> {
        let mut reports = Vec::new();
        for target in self.remotes_permitting(Permission::RemotePull, remote)? {
            let (mut helper, caps) = self.connect(remotes, target)?;
            let since = self.store.sync_token(&target.name)?;
            let step = Step {
                name: "pull",
                remote: Some(&target.name.0),
                doing: &format!("pulling from {}", target.name.0),
                did: &format!("pulled from {}", target.name.0),
                unit: "",
                total: None,
            };
            let response = self.waiting_on(step, || helper.pull(since.as_deref()))?;
            let (mut report, pulled) =
                self.apply_pulled(target, &caps, helper.as_ref(), response)?;
            if since.is_some() {
                report.task_pulled = self.run_pulled_hook(&target.name, pulled)?;
            }
            reports.push(report);
        }
        Ok(reports)
    }

    fn apply_pulled(
        &self,
        remote: &RemoteConfig,
        caps: &RemoteCapabilities,
        helper: &dyn RemoteHelper,
        mut response: PullOutcome,
    ) -> Result<(PullReport, Vec<PulledTask>), UseCaseError> {
        let store = self.store;
        let mut report = PullReport {
            remote: remote.name.clone(),
            created: 0,
            updated: 0,
            conflicts: 0,
            removed_upstream: 0,
            unchanged: 0,
            creates_matched: 0,
            values_kept: 0,
            deletes_confirmed: 0,
            task_pulled: None,
        };
        let listed: HashSet<String> = response
            .objects
            .iter()
            .map(|o| o.remote_id.clone())
            .chain(response.rejected.iter().map(|r| r.remote_id.clone()))
            .collect();
        let mut pulled = Vec::new();
        let mut land_everything_as_one_unit = || {
            record_skipped(store, &remote.name, &response.objects, &response.rejected)?;
            let mut objects = std::mem::take(&mut response.objects);
            objects.extend(cancelled::cancelled_events(
                store,
                remote,
                &response.cancelled,
            )?);
            let (planned, new_remote_id_mappings) = classify(
                store,
                remote,
                caps,
                helper,
                objects,
                self.random,
                &mut report,
            )?;
            for (oid, remote_id) in new_remote_id_mappings {
                store.map_remote_id(&remote.name, &oid, &remote_id)?;
            }
            pulled = self.land(remote, planned, &mut report)?;
            let removed = std::mem::take(&mut response.removed);
            let removals = record_removals(store, remote, removed)?;
            report.removed_upstream = removals.in_the_app;
            report.deletes_confirmed = removals.dams_own;
            if response.complete {
                record_missing(store, remote, &listed)?;
            }
            store.set_sync_token(&remote.name, response.sync.as_deref())?;
            store.set_last_pull(&remote.name, self.clock.now())?;
            Ok(())
        };
        store.in_transaction(&mut land_everything_as_one_unit)?;
        Ok((report, pulled))
    }
}

#[cfg(test)]
mod tests;
