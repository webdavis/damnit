use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Object, Oid};

use crate::config::{Config, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{
    CredentialSource, HelperLauncher, Notice, RemoteHelper, RemoteName, Repositories,
};
use crate::remote::{MutationOp, MutationOutcome, RemoteCapabilities};
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
    config: &Config,
    remote: Option<&str>,
) -> Result<Vec<PushReport>, UseCaseError> {
    let conflicts = repos.conflicts.conflicts()?.len();
    if conflicts > 0 {
        return Err(Refusal::UnresolvedConflicts(conflicts).into());
    }
    let targets = select_remotes(config, remote)?;
    let mut reports = Vec::with_capacity(targets.len());
    for target in targets {
        let (mut helper, caps) = connect(launcher, credentials, target)?;
        reports.push(push_one(repos, helper.as_mut(), &caps, target)?);
    }
    Ok(reports)
}

pub(crate) fn select_remotes<'a>(
    config: &'a Config,
    remote: Option<&str>,
) -> Result<Vec<&'a RemoteConfig>, UseCaseError> {
    match remote {
        None => Ok(config.remotes.iter().collect()),
        Some(name) => config
            .remote(name)
            .map(|r| vec![r])
            .ok_or_else(|| Refusal::NoSuchRemote(name.to_string()).into()),
    }
}

fn push_one(
    repos: Repositories<'_>,
    helper: &mut dyn RemoteHelper,
    caps: &RemoteCapabilities,
    remote: &RemoteConfig,
) -> Result<PushReport, UseCaseError> {
    let unpushed = repos.commits.unpushed(&remote.name)?;
    let changes = changes_to_send(repos.objects, repos.commits, remote, &unpushed)?;

    let mut mutations = Vec::new();
    let mut after_by_oid: BTreeMap<Oid, Object> = BTreeMap::new();
    let mut deletes: BTreeSet<Oid> = BTreeSet::new();
    let mut skipped = 0;
    for (change, origin) in changes.into_values() {
        match mutation_for(
            repos.remote_tracking,
            caps,
            remote,
            &change,
            origin.as_ref(),
        )? {
            Some(mutation) => {
                if mutation.op == MutationOp::Delete {
                    deletes.insert(mutation.oid.clone());
                } else if let Some(after) = &change.after {
                    after_by_oid.insert(change.oid.clone(), after.clone());
                }
                mutations.push(mutation);
            }
            None => skipped += 1,
        }
    }
    let sent = mutations.len();
    let sent_oids: BTreeSet<Oid> = mutations.iter().map(|m| m.oid.clone()).collect();
    let outcomes: Vec<MutationOutcome> = if mutations.is_empty() {
        vec![]
    } else {
        helper.push(mutations)?
    };

    // The helper is an untrusted subprocess: a result for an oid dam never
    // sent is dropped, and a repeated result for one oid keeps only the
    // first (results arrive in the order the helper wrote them).
    let mut failed = Vec::new();
    let mut retries = Vec::new();
    let mut answered: BTreeSet<Oid> = BTreeSet::new();
    for outcome in outcomes {
        let oid = outcome.oid;
        if !sent_oids.contains(&oid) || !answered.insert(oid.clone()) {
            continue;
        }
        if outcome.ok {
            if let Some(id) = &outcome.remote_id {
                repos
                    .remote_tracking
                    .map_remote_id(&remote.name, &oid, id)?;
            }
            if deletes.contains(&oid) {
                repos
                    .remote_tracking
                    .clear_remote_mapping(&remote.name, &oid)?;
            } else if let Some(after) = after_by_oid.get(&oid) {
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, after)?;
            }
        } else {
            let why = outcome
                .why
                .unwrap_or_else(|| "the remote gave no reason".into());
            repos.notices.add_notice(&Notice::PushFailed {
                remote: remote.name.clone(),
                oid: oid.clone(),
                why: why.clone(),
            })?;
            retries.push(oid.clone());
            failed.push((oid, why));
        }
    }

    // A sent mutation with no answer at all, not even a failure, never
    // reached a known state on the remote: treat it the same as an
    // explicit failure rather than silently dropping it.
    let mut unanswered = BTreeSet::new();
    for oid in sent_oids.difference(&answered).cloned() {
        let why = "no answer from helper".to_string();
        repos.notices.add_notice(&Notice::PushFailed {
            remote: remote.name.clone(),
            oid: oid.clone(),
            why: why.clone(),
        })?;
        retries.push(oid.clone());
        unanswered.insert(oid.clone());
        failed.push((oid, why));
    }
    repos.commits.set_push_retries(&remote.name, &retries)?;
    for record in &unpushed {
        if record.changes.iter().any(|c| unanswered.contains(&c.oid)) {
            continue;
        }
        repos.commits.mark_pushed(&remote.name, &record.id)?;
    }
    Ok(PushReport {
        remote: remote.name.clone(),
        sent,
        succeeded: sent.saturating_sub(failed.len()),
        failed,
        skipped,
    })
}

mod idempotency;
mod mutations;

use mutations::{changes_to_send, mutation_for};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_answers;
#[cfg(test)]
mod tests_refusals;
