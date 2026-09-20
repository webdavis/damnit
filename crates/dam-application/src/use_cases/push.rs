use std::collections::{BTreeMap, BTreeSet};

use dam_domain::{Change, CommitId, Object, Oid, Op, changed_fields, coalesce};
use dam_protocol::{Capabilities, Mutation, PushResponse};

use crate::config::{Config, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::ports::{
    CommitRepository, CredentialSource, HelperLauncher, Notice, ObjectRepository, RemoteHelper,
    RemoteName, RemoteTrackingRepository, Repositories,
};
use crate::use_cases::connect::connect;
use crate::use_cases::push::idempotency::{Origin, key};
use crate::wire::to_wire;

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
    caps: &Capabilities,
    remote: &RemoteConfig,
) -> Result<PushReport, UseCaseError> {
    let unpushed = repos.commits.unpushed(&remote.name)?;
    let changes = changes_to_send(repos.objects, repos.commits, remote, &unpushed)?;

    let mut mutations = Vec::new();
    let mut after_by_oid: BTreeMap<String, Object> = BTreeMap::new();
    let mut deletes: BTreeSet<String> = BTreeSet::new();
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
                if mutation.op == "delete" {
                    deletes.insert(mutation.oid.clone());
                } else if let Some(after) = &change.after {
                    after_by_oid.insert(change.oid.to_string(), after.clone());
                }
                mutations.push(mutation);
            }
            None => skipped += 1,
        }
    }
    let sent = mutations.len();
    let sent_oids: BTreeSet<String> = mutations.iter().map(|m| m.oid.clone()).collect();
    let response = if mutations.is_empty() {
        PushResponse { results: vec![] }
    } else {
        helper.push(mutations)?
    };

    // The helper is an untrusted subprocess: a result for an oid dam never
    // sent is dropped, and a repeated result for one oid keeps only the
    // first (results arrive in the order the helper wrote them).
    let mut failed = Vec::new();
    let mut retries = Vec::new();
    let mut answered: BTreeSet<String> = BTreeSet::new();
    for result in response.results {
        if !sent_oids.contains(&result.oid) || !answered.insert(result.oid.clone()) {
            continue;
        }
        let Ok(oid) = Oid::parse(&result.oid) else {
            continue;
        };
        if result.ok {
            if let Some(id) = &result.remote_id {
                repos
                    .remote_tracking
                    .map_remote_id(&remote.name, &oid, id)?;
            }
            if deletes.contains(&result.oid) {
                repos
                    .remote_tracking
                    .clear_remote_mapping(&remote.name, &oid)?;
            } else if let Some(after) = after_by_oid.get(&result.oid) {
                repos
                    .remote_tracking
                    .set_remote_snapshot(&remote.name, after)?;
            }
        } else {
            let why = result
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
    for oid_str in sent_oids.difference(&answered) {
        let Ok(oid) = Oid::parse(oid_str) else {
            continue;
        };
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

/// Every unpushed commit's changes, coalesced per oid and paired with the
/// commit the last of them came from, plus a synthetic update from the
/// committed view for each oid owed a retry that no unpushed commit already
/// covers.
///
/// A coalesced commit change already ends at the committed state, so leaving
/// it in place resends exactly what the synthetic one would have, and keeps
/// the commit that names it. That name is what lets a resend after an
/// interrupted push carry the key the interrupted one carried.
fn changes_to_send(
    objects: &dyn ObjectRepository,
    commits: &dyn CommitRepository,
    remote: &RemoteConfig,
    unpushed: &[dam_domain::CommitRecord],
) -> Result<BTreeMap<Oid, (Change, Option<CommitId>)>, UseCaseError> {
    let mut changes: BTreeMap<Oid, (Change, Option<CommitId>)> = BTreeMap::new();
    for record in unpushed {
        for change in &record.changes {
            let merged = match changes.remove(&change.oid) {
                Some((previous, _)) => coalesce(previous, change.clone()),
                None => Some(change.clone()),
            };
            if let Some(c) = merged {
                changes.insert(c.oid.clone(), (c, Some(record.id.clone())));
            }
        }
    }
    for oid in commits.push_retries(&remote.name)? {
        if changes.contains_key(&oid) {
            continue;
        }
        if let Some(current) = objects.committed(&oid)? {
            changes.insert(
                oid.clone(),
                (
                    Change {
                        oid,
                        op: Op::Update,
                        before: None,
                        after: Some(current),
                    },
                    None,
                ),
            );
        }
    }
    Ok(changes)
}

fn mutation_for(
    remote_tracking: &dyn RemoteTrackingRepository,
    caps: &Capabilities,
    remote: &RemoteConfig,
    change: &Change,
    origin: Option<&CommitId>,
) -> Result<Option<Mutation>, UseCaseError> {
    let Some(object) = change.after.as_ref().or(change.before.as_ref()) else {
        return Ok(None);
    };
    let kind = match object {
        Object::Task(_) => "task",
        Object::Event(_) => "event",
    };
    if !caps.kinds.iter().any(|k| k == kind) {
        return Ok(None);
    }
    if let Some(scope) = &remote.path
        && !object.base().path.is_within(scope)
    {
        return Ok(None);
    }
    let remote_id = remote_tracking.remote_id(&remote.name, &change.oid)?;
    // Whether the remote already holds the object decides the verb, not the
    // change's own op: a create the remote has already taken is an update, and
    // an update of something the remote has never seen is a create.
    let (op, fields): (&str, Vec<String>) = match (change.op, &change.before, &change.after) {
        (Op::Delete, _, _) => ("delete", vec![]),
        (Op::Update, _, None) => return Ok(None),
        (_, _, _) if remote_id.is_none() => ("create", caps.fields.clone()),
        (Op::Create, _, _) => ("update", caps.fields.clone()),
        (Op::Update, Some(before), Some(after)) => {
            let declared: Vec<String> = changed_fields(before, after)
                .into_iter()
                .map(|f| f.as_str().to_string())
                .filter(|f| caps.fields.contains(f))
                .collect();
            if declared.is_empty() {
                return Ok(None);
            }
            ("update", declared)
        }
        (Op::Update, None, Some(_)) => ("update", caps.fields.clone()),
    };
    if op == "delete" && remote_id.is_none() {
        return Ok(None);
    }
    Ok(Some(Mutation {
        op: op.to_string(),
        oid: change.oid.to_string(),
        idempotency_key: key(origin.map_or(Origin::Retry, Origin::Commit), &change.oid),
        remote_id,
        object: if op == "delete" {
            None
        } else {
            change.after.as_ref().map(|a| to_wire(a, None))
        },
        fields,
    }))
}

mod idempotency;

#[cfg(test)]
mod tests;
