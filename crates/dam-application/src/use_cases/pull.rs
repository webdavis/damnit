use dam_domain::{Change, CommitId, CommitRecord, Object, Oid, Op};

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
mod notices;

use classify::{Incoming, classify};
use notices::{note_cancelled, record_removals};

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
        let (mut helper, caps) = connect(launcher, credentials, target)?;
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

fn land(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    random: &dyn Randomness,
    remote: &RemoteConfig,
    planned: Vec<(Oid, Incoming)>,
    report: &mut PullReport,
) -> Result<(), UseCaseError> {
    let mut landed = Vec::new();
    for (oid, incoming) in planned {
        match incoming {
            Incoming::Unchanged => report.unchanged += 1,
            Incoming::Create(o) | Incoming::FastForward(o) => {
                let is_create = repos.objects.get(&oid)?.is_none();
                repos.objects.put(&o)?;
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, &o)?;
                landed.push((
                    oid,
                    if is_create { Op::Create } else { Op::Update },
                    o.clone(),
                ));
                if is_create {
                    report.created += 1
                } else {
                    report.updated += 1
                }
                note_cancelled(repos.objects, repos.notices, &o)?;
            }
            Incoming::Conflict(o) => {
                repos.conflicts.mark_conflict(&remote.name, &oid, &o)?;
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, &o)?;
                report.conflicts += 1;
                note_cancelled(repos.objects, repos.notices, &o)?;
            }
        }
    }
    if landed.is_empty() {
        return Ok(());
    }
    commit_as_already_pushed(repos, clock, random, remote, landed)
}

fn commit_as_already_pushed(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    random: &dyn Randomness,
    remote: &RemoteConfig,
    landed: Vec<(Oid, Op, Object)>,
) -> Result<(), UseCaseError> {
    let record = CommitRecord {
        id: CommitId::generate(&mut |b| random.fill(b)),
        message: format!("pull from {}", remote.name.0),
        at: clock.now(),
        changes: landed
            .into_iter()
            .map(|(oid, op, after)| Change {
                oid,
                op,
                before: None,
                after: Some(after),
            })
            .collect(),
    };
    repos.commits.commit(&record)?;
    repos.commits.mark_pushed(&remote.name, &record.id)?;
    Ok(())
}

#[cfg(test)]
mod tests;
