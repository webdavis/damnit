use crate::config::{Config, RemoteConfig};
use crate::errors::UseCaseError;
use crate::ports::{
    Clock, CredentialSource, HelperLauncher, Notice, Randomness, RemoteName, Repositories,
};
use crate::remote::{PullOutcome, RemoteCapabilities};
use crate::use_cases::connect::connect;
use crate::use_cases::push::select_remotes;

mod cancelled;
mod classify;
mod land;
mod notices;

use classify::classify;
use land::land;
use notices::record_removals;

#[derive(Debug)]
pub struct PullReport {
    pub remote: RemoteName,
    pub created: usize,
    pub updated: usize,
    pub conflicts: usize,
    pub removed_upstream: usize,
    pub unchanged: usize,
}

pub fn pull(
    repos: Repositories<'_>,
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    clock: &dyn Clock,
    random: &dyn Randomness,
    config: &Config,
    remote: Option<&str>,
) -> Result<Vec<PullReport>, UseCaseError> {
    let mut reports = Vec::new();
    for target in select_remotes(config, remote)? {
        let (mut helper, caps) = connect(launcher, credentials, repos.accepted_kinds, target)?;
        let since = repos.remote_tracking.sync_token(&target.name)?;
        let response = helper.pull(since.as_deref())?;
        reports.push(apply(repos, clock, random, target, &caps, response)?);
    }
    Ok(reports)
}

fn apply(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    random: &dyn Randomness,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    mut response: PullOutcome,
) -> Result<PullReport, UseCaseError> {
    let mut report = PullReport {
        remote: remote.name.clone(),
        created: 0,
        updated: 0,
        conflicts: 0,
        removed_upstream: 0,
        unchanged: 0,
    };
    let mut land_everything_as_one_unit = || {
        for rejected in &response.rejected {
            repos.notices.add_notice(&Notice::PullFailed {
                remote: remote.name.clone(),
                why: format!("{}: {}", rejected.remote_id, rejected.why),
            })?;
        }
        let mut objects = std::mem::take(&mut response.objects);
        objects.extend(cancelled::cancelled_events(
            repos,
            remote,
            &response.cancelled,
        )?);
        let (planned, new_remote_id_mappings) = classify(repos, remote, caps, objects, random)?;
        for (oid, remote_id) in new_remote_id_mappings {
            repos
                .remote_tracking
                .map_remote_id(&remote.name, &oid, &remote_id)?;
        }
        land(repos, clock, random, remote, planned, &mut report)?;
        let removed = std::mem::take(&mut response.removed);
        report.removed_upstream = record_removals(repos, remote, removed)?;
        repos
            .remote_tracking
            .set_sync_token(&remote.name, response.sync.as_deref())?;
        repos
            .remote_tracking
            .set_last_pull(&remote.name, clock.now())?;
        Ok(())
    };
    repos
        .transaction
        .in_transaction(&mut land_everything_as_one_unit)?;
    Ok(report)
}

#[cfg(test)]
mod tests;
