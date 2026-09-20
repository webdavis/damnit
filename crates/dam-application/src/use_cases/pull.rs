use dam_domain::{Change, CommitId, CommitRecord, EventStatus, Object, Oid, Op};

use crate::config::{Config, RemoteConfig};
use crate::errors::{Refusal, UseCaseError};
use crate::merge::{kind_change, merge_fields};
use crate::ports::{
    Clock, CredentialSource, HelperLauncher, Notice, NoticeRepository, ObjectRepository,
    Randomness, RemoteName, Repositories,
};
use crate::remote::{IncomingObject, PullOutcome, RemoteCapabilities};
use crate::use_cases::connect::connect;
use crate::use_cases::push::select_remotes;

#[derive(Debug)]
pub struct PullReport {
    pub remote: RemoteName,
    pub created: usize,
    pub updated: usize,
    pub conflicts: usize,
    pub removed_upstream: usize,
    pub unchanged: usize,
}

enum Incoming {
    Create(Object),
    FastForward(Object),
    Conflict(Object),
    Unchanged,
}

/// What each pulled object should do, plus the oids generated for objects
/// with no local match yet, paired with the remote id each one maps to.
type Classified = (Vec<(Oid, Incoming)>, Vec<(Oid, String)>);

pub fn pull(
    repos: Repositories<'_>,
    launcher: &dyn HelperLauncher,
    credentials: &dyn CredentialSource,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
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
    random: &mut dyn Randomness,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    response: PullOutcome,
) -> Result<PullReport, UseCaseError> {
    for rejected in &response.rejected {
        repos.notices.add_notice(&Notice::PullFailed {
            remote: remote.name.clone(),
            why: format!("{}: {}", rejected.remote_id, rejected.why),
        })?;
    }
    let (planned, fresh) = classify(repos, remote, caps, response.objects, random)?;
    for (oid, remote_id) in fresh {
        repos
            .remote_tracking
            .map_remote_id(&remote.name, &oid, &remote_id)?;
    }
    let mut report = PullReport {
        remote: remote.name.clone(),
        created: 0,
        updated: 0,
        conflicts: 0,
        removed_upstream: 0,
        unchanged: 0,
    };
    land(repos, clock, random, remote, planned, &mut report)?;
    report.removed_upstream = record_removals(repos, remote, response.removed)?;
    repos
        .remote_tracking
        .set_sync_token(&remote.name, response.sync.as_deref())?;
    repos
        .remote_tracking
        .set_last_pull(&remote.name, clock.now())?;
    Ok(report)
}

/// Reads every pulled object against local state and decides what to do with
/// it. Notices are the only thing it writes; no object is touched. `fresh`
/// pairs a generated oid with the remote id it maps to, applied by the caller
/// once classification succeeds.
fn classify(
    repos: Repositories<'_>,
    remote: &RemoteConfig,
    caps: &RemoteCapabilities,
    objects: Vec<IncomingObject>,
    random: &mut dyn Randomness,
) -> Result<Classified, UseCaseError> {
    let staged: Vec<Oid> = repos.stage.staged()?.into_iter().map(|c| c.oid).collect();
    let mut notices = repos.notices.notices()?;
    let mut planned = Vec::new();
    let mut fresh = Vec::new();

    for incoming in objects {
        let IncomingObject {
            remote_id,
            mut object,
        } = incoming;
        let existing = repos
            .remote_tracking
            .oid_for_remote_id(&remote.name, &remote_id)?;
        let oid = existing
            .clone()
            .unwrap_or_else(|| Oid::generate(&mut |b| random.fill(b)));
        // The remote knows the object by its own id; the oid it is tracked
        // by here is dam's to supply.
        object.base_mut().oid = oid.clone();
        let theirs_raw = object;
        if existing.is_none() {
            fresh.push((oid.clone(), remote_id));
        }
        let local = repos.objects.get(&oid)?;
        if let Some((ours, theirs)) = local.as_ref().and_then(|l| kind_change(l, &theirs_raw)) {
            let notice = Notice::KindChanged {
                oid: oid.clone(),
                ours,
                theirs,
            };
            // The kinds keep disagreeing until someone acts, so say it once.
            if !notices.contains(&notice) {
                repos.notices.add_notice(&notice)?;
                notices.push(notice);
            }
        }
        let theirs = merge_fields(
            local.as_ref().unwrap_or(&theirs_raw),
            &theirs_raw,
            &caps.fields,
        );
        let incoming = match local {
            None => Incoming::Create(theirs),
            Some(ref l) if *l == theirs => Incoming::Unchanged,
            Some(ref l) => {
                let base = repos.remote_tracking.remote_snapshot(&remote.name, &oid)?;
                if base.as_ref() == Some(l) {
                    Incoming::FastForward(theirs)
                } else if base.as_ref() == Some(&theirs) {
                    // The remote holds what it held when we last looked, so
                    // nothing is coming in; the difference is ours to push.
                    // This is also what stops a conflict already raised, and
                    // settled with ours, from being raised again on every pull
                    // until the resolution reaches the remote.
                    Incoming::Unchanged
                } else {
                    let committed = repos.objects.committed(&oid)?;
                    if committed.as_ref() != Some(l) || staged.contains(&oid) {
                        return Err(Refusal::DirtyOnPull { oid }.into());
                    }
                    Incoming::Conflict(theirs)
                }
            }
        };
        planned.push((oid, incoming));
    }
    Ok((planned, fresh))
}

/// Writes every classified object to the store and commits the batch of
/// creates and fast-forwards as one pull commit, already marked pushed for
/// this remote since it is exactly what the remote holds.
fn land(
    repos: Repositories<'_>,
    clock: &dyn Clock,
    random: &mut dyn Randomness,
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

/// Turns each remote id the remote no longer has into a notice, and drops
/// its oid-to-remote-id mapping and snapshot: the local object stays, but
/// nothing about it still tracks that remote.
fn record_removals(
    repos: Repositories<'_>,
    remote: &RemoteConfig,
    removed: Vec<String>,
) -> Result<usize, UseCaseError> {
    let mut count = 0;
    for remote_id in removed {
        if let Some(oid) = repos
            .remote_tracking
            .oid_for_remote_id(&remote.name, &remote_id)?
        {
            let subject = repos
                .objects
                .get(&oid)?
                .map(|o| o.base().subject.clone())
                .unwrap_or_default();
            repos.notices.add_notice(&Notice::RemovedUpstream {
                remote: remote.name.clone(),
                oid: oid.clone(),
                subject,
            })?;
            repos
                .remote_tracking
                .clear_remote_mapping(&remote.name, &oid)?;
            count += 1;
        }
    }
    Ok(count)
}

fn note_cancelled(
    objects: &dyn ObjectRepository,
    notices: &dyn NoticeRepository,
    object: &Object,
) -> Result<(), UseCaseError> {
    if let Object::Event(e) = object
        && e.status == EventStatus::Cancelled
    {
        let attached = objects
            .all()?
            .iter()
            .filter(|o| {
                o.as_task()
                    .is_some_and(|t| t.event.as_ref() == Some(&e.base.oid))
            })
            .count();
        if attached > 0 {
            notices.add_notice(&Notice::EventCancelled {
                oid: e.base.oid.clone(),
                subject: e.base.subject.clone(),
                attached,
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_kind_change;
#[cfg(test)]
mod tests_removals_and_notices;
