use dam_domain::{CommitRecord, Oid};

use crate::config::{Config, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{
    Clock, CredentialSource, HelperLauncher, Notice, RemoteHelper, RemoteName, Repositories,
};
use crate::remote::RemoteCapabilities;
use crate::use_cases::atomically::atomically;
use crate::use_cases::connect::connect;

#[derive(Debug)]
pub struct PushReport {
    pub remote: RemoteName,
    pub sent: usize,
    pub succeeded: usize,
    pub failed: Vec<(Oid, String)>,
    pub skipped: usize,
}

pub fn push(
    repos: Repositories<'_>,
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    clock: &dyn Clock,
    config: &Config,
    remote: Option<&str>,
) -> Result<Vec<PushReport>, UseCaseError> {
    let conflicts = repos.conflicts.conflicts()?.len();
    if conflicts > 0 {
        return Err(Refusal::UnresolvedConflicts(conflicts).into());
    }
    let targets = config.select_remotes(remote)?;
    let mut reports = Vec::with_capacity(targets.len());
    for target in targets {
        let (mut helper, caps) = connect(launcher, credentials, repos.accepted_kinds, target)?;
        reports.push(push_one(repos, helper.as_mut(), &caps, clock, target)?);
    }
    Ok(reports)
}

fn push_one(
    repos: Repositories<'_>,
    helper: &mut dyn RemoteHelper,
    caps: &RemoteCapabilities,
    clock: &dyn Clock,
    remote: &RemoteConfig,
) -> Result<PushReport, UseCaseError> {
    let unpushed = repos.commits.unpushed(&remote.name)?;
    let changes = changes_to_send(
        repos.objects,
        repos.commits,
        repos.remote_tracking,
        remote,
        &unpushed,
    )?;
    let Plan { mutations, skipped } = plan(repos.remote_tracking, caps, remote, changes)?;
    let outcomes = if mutations.is_empty() {
        vec![]
    } else {
        helper.push(mutations.clone())?
    };
    let settlement = reconcile(&mutations, outcomes);
    atomically(repos.transaction, || {
        apply(repos, clock, remote, &unpushed, &settlement)
    })?;
    let sent = mutations.len();
    Ok(PushReport {
        remote: remote.name.clone(),
        sent,
        succeeded: sent.saturating_sub(settlement.failed.len()),
        failed: settlement.failed,
        skipped,
    })
}

fn apply(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    remote: &RemoteConfig,
    unpushed: &[CommitRecord],
    settlement: &Settlement,
) -> Result<(), UseCaseError> {
    let tracking = repos.remote_tracking;
    for (oid, id) in &settlement.map {
        tracking.map_remote_id(&remote.name, oid, id)?;
    }
    for oid in &settlement.clear {
        tracking.clear_remote_mapping(&remote.name, oid)?;
    }
    for after in &settlement.snapshot {
        tracking.set_remote_snapshot(&remote.name, after)?;
    }
    for (oid, why) in &settlement.failed {
        repos.notices.add_notice(&Notice::PushFailed {
            remote: remote.name.clone(),
            oid: oid.clone(),
            why: why.clone(),
        })?;
    }
    repos
        .commits
        .set_push_retries(&remote.name, &settlement.retries())?;
    for id in settlement.pushed(unpushed) {
        repos.commits.mark_pushed(&remote.name, id)?;
    }
    tracking.set_last_push(&remote.name, clock.now())?;
    Ok(())
}

mod idempotency;
mod mutations;
mod reconcile;

use mutations::{Plan, changes_to_send, plan};
use reconcile::{Settlement, reconcile};

#[cfg(test)]
mod tests;
